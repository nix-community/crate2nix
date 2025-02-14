//! Utilities for calling `nix-prefetch` on packages.

use std::io::Write;
use std::process::Command;

use crate::metadata::PackageIdShortener;
use crate::resolve::{CrateDerivation, CratesIoSource, GitSource, RegistrySource, ResolvedSource};
use crate::GenerateConfig;
use anyhow::bail;
use anyhow::format_err;
use anyhow::Error;
use cargo_metadata::PackageId;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};

/// The source is important because we need to store only hashes for which we performed
/// a prefetch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HashSource {
    Prefetched,
    Existing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HashWithSource {
    sha256: String,
    source: HashSource,
}

/// A source with all the packages that depend on it and a potentially preexisting hash.
#[derive(Debug)]
struct SourcePrefetchBundle<'a> {
    source: &'a ResolvedSource,
    packages: &'a Vec<&'a CrateDerivation>,
    hash: Option<HashWithSource>,
}

/// Uses `nix-prefetch` to get the hashes of the sources for the given packages if they come from crates.io.
///
/// Uses and updates the existing hashes in the `config.crate_hash_json` file.
pub fn prefetch(
    config: &GenerateConfig,
    from_lock_file: &HashMap<PackageId, String>,
    crate_derivations: &[CrateDerivation],
    id_shortener: &PackageIdShortener,
) -> Result<BTreeMap<PackageId, String>, Error> {
    let hashes_string: String = if config.read_crate_hashes {
        std::fs::read_to_string(&config.crate_hashes_json).unwrap_or_else(|_| "{}".to_string())
    } else {
        "{}".to_string()
    };

    let old_prefetched_hashes: BTreeMap<PackageId, String> = serde_json::from_str(&hashes_string)?;

    // Only copy used hashes over to the new map.
    let mut hashes = BTreeMap::<PackageId, String>::new();

    // Multiple packages might be fetched from the same source.
    //
    // Usually, a source is only used by one package but e.g. the same git source can be used
    // by multiple packages.
    let packages_by_source: HashMap<ResolvedSource, Vec<&CrateDerivation>> = {
        let mut index = HashMap::new();
        for package in crate_derivations {
            index
                .entry(package.source.without_sha256())
                .or_insert_with(Vec::new)
                .push(package);
        }
        index
    };

    // Associate prefetchable sources with existing hashes.
    let prefetchable_sources: Vec<SourcePrefetchBundle> = packages_by_source
        .iter()
        .filter(|(source, _)| source.needs_prefetch())
        .map(|(source, packages)| {
            // All the packages have the same source.
            // So is there any package for which we already know the hash?
            let hash = packages
                .iter()
                .filter_map(|p| {
                    from_lock_file
                        .get(&p.package_id)
                        .map(|hash| HashWithSource {
                            sha256: hash.clone(),
                            source: HashSource::Existing,
                        })
                        .or_else(|| {
                            old_prefetched_hashes
                                .get(id_shortener.lengthen_ref(&p.package_id))
                                .map(|hash| HashWithSource {
                                    sha256: hash.clone(),
                                    source: HashSource::Prefetched,
                                })
                        })
                        .or_else(|| {
                            // This happens e.g. if the sha256 comes from crate2nix.json.
                            packages
                                .iter()
                                .filter_map(|p| p.source.sha256())
                                .next()
                                .map(|hash| HashWithSource {
                                    sha256: hash.clone(),
                                    source: HashSource::Existing,
                                })
                        })
                })
                .next();

            SourcePrefetchBundle {
                source,
                packages,
                hash,
            }
        })
        .collect();

    let without_hash_num = prefetchable_sources
        .iter()
        .filter(|SourcePrefetchBundle { hash, .. }| hash.is_none())
        .count();

    let mut idx = 1;
    for SourcePrefetchBundle {
        source,
        packages,
        hash,
    } in prefetchable_sources
    {
        let (sha256, hash_source) = if let Some(HashWithSource { sha256, source }) = hash {
            (sha256.trim().to_string(), source)
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

        for package in packages {
            if hash_source == HashSource::Prefetched {
                hashes.insert(
                    id_shortener.lengthen_ref(&package.package_id).clone(),
                    sha256.clone(),
                );
            }
        }
    }

    if hashes != old_prefetched_hashes {
        std::fs::write(
            &config.crate_hashes_json,
            serde_json::to_vec_pretty(&hashes)?,
        )
        .map_err(|e| {
            format_err!(
                "while writing hashes to {}: {}",
                config.crate_hashes_json.to_str().unwrap_or("<unknown>"),
                e
            )
        })?;
        eprintln!(
            "Wrote hashes to {}.",
            config.crate_hashes_json.to_string_lossy()
        );
    }

    Ok(hashes)
}

/// Prefetch the config.json file from all the derivation's private registries.
pub fn prefetch_registries(
    config: &GenerateConfig,
    crate_derivations: &[CrateDerivation],
) -> Result<BTreeMap<String, String>, Error> {
    let hashes_string: String = if config.read_crate_hashes {
        std::fs::read_to_string(&config.registry_hashes_json).unwrap_or_else(|_| "{}".to_string())
    } else {
        "{}".to_string()
    };

    let old_prefetched_hashes: BTreeMap<String, String> = serde_json::from_str(&hashes_string)?;

    let mut hashes = old_prefetched_hashes.clone();

    for package in crate_derivations {
        let registry =
            if let ResolvedSource::Registry(RegistrySource { ref registry, .. }) = package.source {
                registry
            } else {
                continue;
            };
        use std::collections::btree_map::Entry;
        if let Entry::Vacant(e) = hashes.entry(registry.to_string()) {
            eprintln!("Prefetching {} config", e.key());
            let out = get_command_output(
                "nix-prefetch-url",
                &[&format!(
                    "{}{}config.json",
                    e.key(),
                    if e.key().ends_with('/') { "" } else { "/" }
                )],
            )?;
            e.insert(out);
        }
    }

    if hashes != old_prefetched_hashes {
        std::fs::write(
            &config.registry_hashes_json,
            serde_json::to_vec_pretty(&hashes)?,
        )
        .map_err(|e| {
            format_err!(
                "while writing hashes to {}: {}",
                config.crate_hashes_json.to_str().unwrap_or("<unknown>"),
                e
            )
        })?;
        eprintln!(
            "Wrote hashes to {}.",
            config.registry_hashes_json.to_string_lossy()
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

/// A crate source that potentially has a prefetchable hash.
pub trait PrefetchableSource: ToString {
    /// Returns whether we actually need a prefetch. `false` if
    /// e.g. we already have the hash.
    fn needs_prefetch(&self) -> bool;
    /// Prefetches the source and returns the hash.
    fn prefetch(&self) -> Result<String, Error>;
}

impl ResolvedSource {
    fn inner_prefetchable(&self) -> Option<&dyn PrefetchableSource> {
        match self {
            ResolvedSource::CratesIo(source) => Some(source),
            ResolvedSource::Registry(source) => Some(source),
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

impl PrefetchableSource for RegistrySource {
    fn needs_prefetch(&self) -> bool {
        self.sha256.is_none()
    }

    fn prefetch(&self) -> Result<String, Error> {
        // This is done in two steps, currently only implemented in
        // the generated Nix.
        unimplemented!()
    }
}

impl PrefetchableSource for GitSource {
    fn needs_prefetch(&self) -> bool {
        // self.rev is sufficient for reproducible fetching, and that field is mandatory
        false
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
            self.rev.as_ref(),
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
