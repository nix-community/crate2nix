//! Utilities for calling `nix-prefetch` on packages.

use std::io::Write;
use std::process::{Command, Child, Output, Stdio};

use crate::resolve::{CrateDerivation, CratesIoSource, GitSource, ResolvedSource};
use crate::GenerateConfig;
use cargo_metadata::PackageId;
use failure::bail;
use failure::format_err;
use failure::Error;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap, VecDeque};

/// The source is important because we need to store only hashes for which we performed
/// a prefetch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HashSource {
    Prefetched,
    CargoLock,
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
) -> Result<BTreeMap<PackageId, String>, Error> {
    let hashes_string: String =
        std::fs::read_to_string(&config.crate_hashes_json).unwrap_or_else(|_| "{}".to_string());

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
                            source: HashSource::CargoLock,
                        })
                        .or_else(|| {
                            old_prefetched_hashes
                                .get(&p.package_id)
                                .map(|hash| HashWithSource {
                                    sha256: hash.clone(),
                                    source: HashSource::Prefetched,
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

    let mut currently_fetching: VecDeque<(&ResolvedSource, &Vec<&CrateDerivation>, _)> = Default::default();
    let mut idx = 1;
    for SourcePrefetchBundle { source, packages, hash } in prefetchable_sources {
        // Wait for previously scheduled prefetching to complete
        if currently_fetching.len() >= config.jobs {
            let (src, defer_pkgs, defer) = currently_fetching.pop_front().unwrap();
            let sha256 = src.finish_prefetch(defer)?;
            for pkg in defer_pkgs {
                hashes.insert(pkg.package_id.clone(), sha256.clone());
            }
        }

        let result = if let Some(HashWithSource { sha256, source: hash_source }) = hash {
            let sha256 = sha256.trim().to_string();
            for package in packages {
                if hash_source == HashSource::Prefetched {
                    hashes.insert(package.package_id.clone(), sha256.clone());
                }
            }
        } else {
            eprintln!("Prefetching {:>4}/{}: {}", idx, without_hash_num, source.to_string());
            idx += 1;
            currently_fetching.push_back((source, packages, source.start_prefetch()?));
        };

    }
    while !currently_fetching.is_empty() {
        let (src, defer_pkgs, defer) = currently_fetching.pop_front().unwrap();
        let sha256 = src.finish_prefetch(defer)?;
        for pkg in defer_pkgs {
            hashes.insert(pkg.package_id.clone(), sha256.clone());
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

fn get_command_output(command: Command, output: Output) -> Result<String, Error> {
    if !output.status.success() {
        std::io::stdout().write_all(&output.stdout)?;
        std::io::stderr().write_all(&output.stderr)?;
        bail!(
            "{:?}\n=> exited with: {}",
            command,
            output.status.code().unwrap_or(-1)
        );
    }

    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_string())
        .map_err(|_e| format_err!("output of {:?} is not UTF8!", command))
}

trait PrefetchableSource: ToString {
    type StartResult;
    fn needs_prefetch(&self) -> bool;
    fn start_prefetch(&self) -> Result<Self::StartResult, Error>;
    fn finish_prefetch(&self, defer: Self::StartResult) -> Result<String, Error>;
}

impl PrefetchableSource for ResolvedSource {
    type StartResult = (Command, Child);

    fn needs_prefetch(&self) -> bool {
        match self {
            ResolvedSource::CratesIo(source) => source.needs_prefetch(),
            ResolvedSource::Git(source) => source.needs_prefetch(),
            ResolvedSource::LocalDirectory(..) => false,
        }
    }

    fn start_prefetch(&self) -> Result<Self::StartResult, Error> {
        match self {
            ResolvedSource::CratesIo(source) => source.start_prefetch(),
            ResolvedSource::Git(source) => source.start_prefetch(),
            ResolvedSource::LocalDirectory(..) =>
                Err(format_err!("source does not support prefetch: {:?}", self)),
        }
    }

    fn finish_prefetch(&self, child: Self::StartResult) -> Result<String, Error> {
        match self {
            ResolvedSource::CratesIo(source) => source.finish_prefetch(child),
            ResolvedSource::Git(source) => source.finish_prefetch(child),
            ResolvedSource::LocalDirectory(..) => unreachable!(),
        }
    }
}

impl PrefetchableSource for CratesIoSource {
    type StartResult = (Command, Child);

    fn needs_prefetch(&self) -> bool {
        self.sha256.is_none()
    }

    fn start_prefetch(&self) -> Result<Self::StartResult, Error> {
        let args = &[
            &self.url(),
            "--name",
            &format!("{}-{}", self.name, self.version),
        ];
        let mut command = Command::new("nix-prefetch-url");
        command.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());
        let child = command.spawn()?;
        Ok((command, child))
    }

    fn finish_prefetch(&self, child: Self::StartResult) -> Result<String, Error> {
        get_command_output(child.0, child.1.wait_with_output()?)
    }
}

impl PrefetchableSource for GitSource {
    type StartResult = (Command, Child);

    fn needs_prefetch(&self) -> bool {
        self.sha256.is_none()
    }

    fn start_prefetch(&self) -> Result<Self::StartResult, Error> {
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
        let mut command = Command::new("nix-prefetch-git");
        command.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());
        let child = command.spawn()?;
        Ok((command, child))
    }

    fn finish_prefetch(&self, child: Self::StartResult) -> Result<String, Error> {
        /// A struct used to contain the output returned by `nix-prefetch-git`.
        ///
        /// Additional fields are available (e.g., `name`), but we only call `nix-prefetch-git` to
        /// obtain the nix sha256 for use in calls to `pkgs.fetchgit` in generated `Cargo.nix`
        /// files so there's no reason to declare the fields here until they are needed.
        #[derive(Deserialize)]
        struct NixPrefetchGitInfo {
            sha256: String,
        }

        let json = get_command_output(child.0, child.1.wait_with_output()?)?;
        let prefetch_info: NixPrefetchGitInfo = serde_json::from_str(&json)?;
        Ok(prefetch_info.sha256)
    }


}
