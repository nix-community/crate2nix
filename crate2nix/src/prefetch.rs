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
struct SourcePrefetchBundle {
    source: ResolvedSource,
    derivations: Vec<usize>,
    hash: Option<HashWithSource>,
}

pub(crate) struct Prefetcher<'a> {
    config: &'a GenerateConfig,
    from_lock_file: &'a HashMap<PackageId, String>,
    crate_derivations: &'a [CrateDerivation],
    hashes: BTreeMap<PackageId, String>,
    fetch_queue: VecDeque<(ResolvedSource, Vec<PackageId>, (Command, Child))>,
    started_idx: usize,
}

impl<'a> Prefetcher<'a> {
    pub(crate) fn new(
        config: &'a GenerateConfig,
        from_lock_file: &'a HashMap<PackageId, String>,
        crate_derivations: &'a [CrateDerivation],
    ) -> Prefetcher<'a> {
        Prefetcher {
            config,
            from_lock_file,
            crate_derivations,
            hashes: Default::default(),
            fetch_queue: Default::default(),
            started_idx: 0,
        }
    }

    fn prefetchable_sources(&self, old_prefetches: &BTreeMap<PackageId, String>)
    -> Vec<SourcePrefetchBundle> {
        // Multiple packages might be fetched from the same source.
        //
        // Usually, a source is only used by one package but e.g. the same git source can be used
        // by multiple packages.
        let packages_by_source = {
            let mut map = HashMap::new();
            for (idx, package) in self.crate_derivations.into_iter().enumerate() {
                map
                    .entry(package.source.without_sha256())
                    .or_insert_with(Vec::new)
                    .push(idx);
            }
            map
        };

        // Associate prefetchable sources with existing hashes.
        packages_by_source
            .into_iter()
            .filter(|(source, _)| source.needs_prefetch())
            .map(|(source, derivations)| {
                // All the packages have the same source.
                // So is there any package for which we already know the hash?
                let hash = derivations
                    .iter()
                    .filter_map(|&drv_idx| {
                        let pkg_id = self.package_id_for(drv_idx);
                        self.from_lock_file
                            .get(&pkg_id)
                            .map(|hash| HashWithSource {
                                sha256: hash.clone(),
                                source: HashSource::CargoLock,
                            })
                            .or_else(|| {
                                old_prefetches
                                    .get(&pkg_id)
                                    .map(|hash| HashWithSource {
                                        sha256: hash.clone(),
                                        source: HashSource::Prefetched,
                                    })
                            })
                    })
                    .next();

                SourcePrefetchBundle {
                    source,
                    derivations,
                    hash,
                }
            })
            .collect()
    }

    /// Uses `nix-prefetch` to get the hashes of the sources for the given packages if they come
    /// from crates.io.
    ///
    /// Uses and updates the existing hashes in the `config.crate_hash_json` file.
    pub(crate) fn prefetch(mut self) -> Result<BTreeMap<PackageId, String>, Error> {
        let hashes_string = std::fs::read_to_string(&self.config.crate_hashes_json)
            .unwrap_or_else(|_| "{}".to_string());
        let old_prefetched_hashes =
            serde_json::from_str(&hashes_string)?;

        {
        let prefetchable_sources = self.prefetchable_sources(&old_prefetched_hashes);

            let without_hash_num = prefetchable_sources
                .iter()
                .filter(|SourcePrefetchBundle { hash, .. }| hash.is_none())
                .count();

            for SourcePrefetchBundle { source, derivations, hash } in prefetchable_sources {
                let result = if let Some(HashWithSource { sha256, source: hash_source }) = hash {
                    let sha256 = sha256.trim().to_string();
                    for drv_idx in derivations {
                        if hash_source == HashSource::Prefetched {
                            self.hashes.insert(self.package_id_for(drv_idx), sha256.clone());
                        }
                    }
                } else {
                    self.enqueue(without_hash_num, source, &derivations)?;
                };

            }
        }

        while !self.fetch_queue.is_empty() {
            self.dequeue();
        }

        if self.hashes != old_prefetched_hashes {
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

        Ok(self.hashes)
    }

    fn package_id_for(&self, derivation_index: usize) -> PackageId {
        self.crate_derivations[derivation_index].package_id.clone()
    }

    fn enqueue(
        &mut self,
        total: usize,
        source: ResolvedSource,
        derivations: &[usize]
    ) -> Result<(), Error> {
        if self.fetch_queue.len() >= self.config.jobs {
            self.dequeue()?;
        }
        self.started_idx += 1;
        eprintln!("Prefetching {:>4}/{}: {}", self.started_idx, total, source.to_string());
        let pkg_ids = derivations.iter()
            .map(|&idx| self.crate_derivations[idx].package_id.clone())
            .collect();
        let defer = source.start_prefetch()?;
        self.fetch_queue.push_back((source, pkg_ids, defer));
        Ok(())
>>>>>>> ae478ae... Split out prefetch into multple methods on a ctx
    }

    fn dequeue(&mut self) -> Result<(), Error> {
        let (src, pkg_ids, defer) = self.fetch_queue.pop_front().unwrap();
        let sha256 = src.finish_prefetch(defer)?;
        for id in pkg_ids {
            self.hashes.insert(id, sha256.clone());
        }
        Ok(())
    }
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
