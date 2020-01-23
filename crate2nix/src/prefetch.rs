//! Utilities for calling `nix-prefetch` on packages.

use std::io::Write;
use std::process::Command;

use crate::resolve::{CrateDerivation, GitSource, ResolvedSource, CratesIoSource};
use crate::GenerateConfig;
use cargo_metadata::PackageId;
use failure::bail;
use failure::format_err;
use failure::Error;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};

/// Uses `nix-prefetch` to get the hashes of the sources for the given packages if they come from crates.io.
///
/// Uses and updates the existing hashes in the `config.crate_hash_json` file.
pub fn prefetch(
    config: &GenerateConfig,
    crate_derivations: &mut [CrateDerivation],
) -> Result<BTreeMap<PackageId, String>, Error> {
    let hashes_string: String =
        std::fs::read_to_string(&config.crate_hashes_json).unwrap_or_else(|_| "{}".to_string());

    let old_prefetched_hashes: BTreeMap<PackageId, String> = serde_json::from_str(&hashes_string)?;

    // Only copy used hashes over to the new map.
    let mut hashes = BTreeMap::<PackageId, String>::new();

    // Multiple packages might be fetched from the same source.
    let mut packages_by_source: HashMap<ResolvedSource, Vec<&mut CrateDerivation>> = {
        let mut index = HashMap::new();
        for package in crate_derivations {
            index
                .entry(package.source.without_sha256())
                .or_insert_with(Vec::new)
                .push(package);
        }
        index
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum HashSource {
        Prefetched,
        OtherPackage,
    }

    // Associate sources with existing hashes.
    let mut prefetchable_sources: Vec<(
        &ResolvedSource,
        &mut Vec<&mut CrateDerivation>,
        Option<(String, HashSource)>,
    )> = packages_by_source
        .iter_mut()
        .filter(|(source, _)| source.needs_prefetch())
        .map(|(source, packages)| {
            // All the packages have the same source.
            // So is there any package for which we already know the hash?
            let existing_hash = packages
                .iter()
                .filter_map(|p| {
                    p.source
                        .sha256()
                        .map(|s| (s.clone(), HashSource::OtherPackage))
                        .or_else(|| {
                            old_prefetched_hashes
                                .get(&p.package_id)
                                .map(|hash| (hash.clone(), HashSource::Prefetched))
                        })
                })
                .next();

            (source, packages, existing_hash)
        })
        .collect();

    let without_hash_num = prefetchable_sources
        .iter()
        .filter(|(_, _, existing_hash)| existing_hash.is_none())
        .count();

    let mut idx = 1;
    for (source, packages, existing_hash) in prefetchable_sources.iter_mut() {
        let (sha256, hash_source) = if let Some((hash, hash_source)) = existing_hash {
            (hash.trim().to_string(), *hash_source)
        } else {
            eprintln!(
                "Prefetching {:>4}/{}: {}",
                idx,
                without_hash_num,
                source.to_string()
            );
            idx += 1;
            (source.prefetch()?, HashSource::Prefetched)
        };

        for package in packages.iter_mut() {
            package.source = package.source.with_sha256(sha256.clone());
            if hash_source == HashSource::Prefetched {
                hashes.insert(package.package_id.clone(), sha256.clone());
            }
        }
    }

    if hashes != old_prefetched_hashes {
        std::fs::write(
            &config.crate_hashes_json,
            serde_json::to_vec_pretty(&hashes)?,
        )?;
        eprintln!(
            "Wrote hashes to {}.",
            config.crate_hashes_json.to_string_lossy()
        );
    }

    Ok(hashes)
}

fn get_command_output(cmd: &str, args: &[&str]) -> Result<String, Error> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format_err!("While spawning '{} {}': {}", cmd, args.join(" "), e))?;

    if !output.status.success() {
        std::io::stdout().write_all(&output.stdout)?;
        std::io::stderr().write_all(&output.stderr)?;
        bail!(
            "{}\n=> exited with: {}",
            cmd,
            output.status.code().unwrap_or(-1)
        );
    }

    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_string())
        .map_err(|_e| format_err!("output of '{} {}' is not UTF8!", cmd, args.join(" ")))
}

trait PrefetchableSource: ToString {
    fn needs_prefetch(&self) -> bool;
    fn prefetch(&self) -> Result<String, Error>;
}

impl ResolvedSource {
    fn inner_prefetchable(&self) -> Option<&dyn PrefetchableSource> {
        match self {
            ResolvedSource::CratesIo(source) => Some(source),
            ResolvedSource::Git(source) => Some(source),
            _ => None,
        }
    }
}

impl PrefetchableSource for ResolvedSource {
    fn needs_prefetch(&self) -> bool {
        self.inner_prefetchable()
            .map(|s| s.needs_prefetch())
            .unwrap_or(false)
    }

    fn prefetch(&self) -> Result<String, Error> {
        self.inner_prefetchable()
            .map(|s| s.prefetch())
            .unwrap_or_else(|| Err(format_err!("source does not support prefetch: {:?}", self)))
    }
}

impl PrefetchableSource for CratesIoSource {
    fn needs_prefetch(&self) -> bool {
        self.sha256.is_none()
    }

    fn prefetch(&self) -> Result<String, Error> {
        let args = &[
            &self.url(),
            "--name",
            &format!("{}-{}", self.name, self.version),
        ];
        get_command_output("nix-prefetch-url", args)
    }
}

impl PrefetchableSource for GitSource {
    fn needs_prefetch(&self) -> bool {
        self.sha256.is_none()
    }

    fn prefetch(&self) -> Result<String, Error> {
        /// A struct used to contain the output returned by `nix-prefetch-git`.
        ///
        /// Additional fields are available (e.g., `name`), but we only call `nix-prefetch-git` to obtain
        /// the nix sha256 for use in calls to `pkgs.fetchgit` in generated `Cargo.nix` files so there's no
        /// reason to declare the fields here until they are needed.
        #[derive(Deserialize)]
        struct NixPrefetchGitInfo {
            sha256: String,
        }

        let mut args = vec![
            "--url",
            self.url.as_str(),
            "--fetch-submodules",
            "--rev",
            &self.rev,
        ];

        // TODO: --branch-name isn't documented in nix-prefetch-git --help
        // TODO: Consider the case when ref *isn't* a branch. You have to pass
        // that to `--rev` instead. This seems like limitation of nix-prefetch-git.
        if let Some(r#ref) = self.r#ref.as_ref() {
            args.extend_from_slice(&["--branch-name", r#ref]);
        }

        let json = get_command_output("nix-prefetch-git", &args)?;
        let prefetch_info: NixPrefetchGitInfo = serde_json::from_str(&json)?;
        Ok(prefetch_info.sha256)
    }
}
