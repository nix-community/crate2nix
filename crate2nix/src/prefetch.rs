//! Utilities for calling `nix-prefetch` on packages.

use crate::resolve::{CrateDerivation, CratesIoSource, GitSource, ResolvedSource};
use crate::GenerateConfig;
use async_trait::async_trait;
use cargo_metadata::PackageId;
use failure::bail;
use failure::format_err;
use failure::Error;
use futures::{StreamExt, TryStreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, HashMap},
    convert::TryInto,
    fs,
    sync::Arc,
};
use tokio::{
    io::{self, AsyncWriteExt},
    process::Command,
};

/// The source is important because we need to store only hashes for which we performed
/// a prefetch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HashSource {
    Prefetched,
    CargoLock,
}

struct HashWithSource {
    sha256: String,
    source: HashSource,
}

/// A source with all the packages that depend on it and a potentially preexisting hash.
struct SourcePrefetchBundle<'a> {
    source: &'a ResolvedSource,
    packages: &'a mut Vec<&'a mut CrateDerivation>,
    hash: Option<HashWithSource>,
}

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
    //
    // Usually, a source is only used by one package but e.g. the same git source can be used
    // by multiple packages.
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

    // Associate prefetchable sources with existing hashes.
    let prefetchable_sources: Vec<_> = packages_by_source
        .iter_mut()
        .filter(|(source, _)| source.needs_prefetch())
        .map(|(source, packages)| {
            let hash = packages
                .iter()
                .filter_map(|p| {
                    p.source
                        .sha256()
                        .map(|s| HashWithSource {
                            sha256: s.clone(),
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

    let num_crates_without_hash = prefetchable_sources
        .iter()
        .fold(0, |acc, SourcePrefetchBundle { hash, .. }| {
            acc + hash.is_none() as usize
        });

    let progress_bar = Arc::new(
        ProgressBar::new(num_crates_without_hash.try_into()?).with_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
                .progress_chars("#>-"),
        ),
    );
    let tasks = prefetchable_sources.into_iter().map(
        |SourcePrefetchBundle {
             source,
             packages,
             hash,
         }| {
            let pb = progress_bar.clone();
            async move {
                let (sha256, hash_source) = if let Some(HashWithSource { sha256, source }) = hash {
                    (sha256.trim().to_string(), source)
                } else {
                    let prefetched_hash = source.prefetch().await?;
                    pb.inc(1);
                    (prefetched_hash, HashSource::Prefetched)
                };

                Result::<_, Error>::Ok(packages.iter_mut().map(move |package| {
                    let package_id = package.package_id.clone();
                    let package_source = package.source.with_sha256(sha256.clone());
                    (
                        package,
                        package_source,
                        if hash_source == HashSource::Prefetched {
                            Some((package_id, sha256.clone()))
                        } else {
                            None
                        },
                    )
                }))
            }
        },
    );

    // TODO: Is there a better way to choose this number?
    let n_concurrent_tasks = num_cpus::get() * 10;
    let bundles: Vec<_> = tokio::runtime::Runtime::new()?.block_on(
        futures::stream::iter(tasks)
            .buffer_unordered(n_concurrent_tasks)
            .try_collect(),
    )?;

    for (package, source, package_hashes) in bundles.into_iter().flatten() {
        package.source = source;
        if let Some((package_id, sha256)) = package_hashes {
            hashes.insert(package_id, sha256);
        }
    }

    if hashes != old_prefetched_hashes {
        fs::write(
            &config.crate_hashes_json,
            serde_json::to_vec_pretty(&hashes)?,
        )?;
        eprintln!("Wrote hashes to {}.", config.crate_hashes_json.display());
    }

    Ok(hashes)
}

async fn get_command_output(cmd: &str, args: &[&str]) -> Result<String, Error> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .await
        .map_err(|e| format_err!("While spawning '{} {}': {}", cmd, args.join(" "), e))?;

    if !output.status.success() {
        io::stdout().write_all(&output.stdout).await?;
        io::stderr().write_all(&output.stderr).await?;
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

#[async_trait]
trait PrefetchableSource {
    fn needs_prefetch(&self) -> bool;
    async fn prefetch(&self) -> Result<String, Error>;
}

impl ResolvedSource {
    fn inner_prefetchable(&self) -> Option<&(dyn PrefetchableSource + Send + Sync)> {
        match self {
            ResolvedSource::CratesIo(source) => Some(source),
            ResolvedSource::Git(source) => Some(source),
            _ => None,
        }
    }
}

#[async_trait]
impl PrefetchableSource for ResolvedSource {
    fn needs_prefetch(&self) -> bool {
        self.inner_prefetchable()
            .map(|s| s.needs_prefetch())
            .unwrap_or(false)
    }

    async fn prefetch(&self) -> Result<String, Error> {
        if let Some(prefetchable) = self.inner_prefetchable() {
            Some(prefetchable.prefetch().await)
        } else {
            None
        }
        .unwrap_or_else(|| Err(format_err!("source does not support prefetch: {:?}", self)))
    }
}

#[async_trait]
impl PrefetchableSource for CratesIoSource {
    fn needs_prefetch(&self) -> bool {
        self.sha256.is_none()
    }

    async fn prefetch(&self) -> Result<String, Error> {
        let args = &[
            &self.url(),
            "--name",
            &format!("{}-{}", self.name, self.version),
        ];
        get_command_output("nix-prefetch-url", args).await
    }
}

#[async_trait]
impl PrefetchableSource for GitSource {
    fn needs_prefetch(&self) -> bool {
        self.sha256.is_none()
    }

    async fn prefetch(&self) -> Result<String, Error> {
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

        // TODO: Consider the case when ref *isn't* a branch. You have to pass
        // that to `--rev` instead. This seems like limitation of nix-prefetch-git.
        if let Some(r#ref) = self.r#ref.as_ref() {
            args.extend_from_slice(&["--branch-name", r#ref]);
        }

        let json = get_command_output("nix-prefetch-git", &args).await?;
        let prefetch_info: NixPrefetchGitInfo = serde_json::from_str(&json)?;
        Ok(prefetch_info.sha256)
    }
}
