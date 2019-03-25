//! # crate2nix
//!
//! Internal library for the crate2nix binary. This is not meant to be used separately, I just enjoy
//! writing doc tests ;)

//#![deny(missing_docs)]

use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;

use cargo_metadata::Metadata;
use cargo_metadata::PackageId;
use failure::format_err;
use failure::Error;
use serde_derive::Deserialize;
use serde_derive::Serialize;

use crate::metadata::IndexedMetadata;
use crate::resolve::CrateDerivation;

mod metadata;
pub mod nix_build;
mod prefetch;
pub mod render;
mod resolve;
mod target_cfg;
pub mod util;

/// The resolved build info and the input for rendering the build.nix.tera template.
#[derive(Debug, Deserialize, Serialize)]
pub struct BuildInfo {
    pub root_package_id: Option<PackageId>,
    pub crates: Vec<CrateDerivation>,
    // For convenience include the source for tests.
    pub indexed_metadata: IndexedMetadata,
    // The generation configuration.
    pub info: GenerateInfo,
    // The generation configuration.
    pub config: GenerateConfig,
}

impl BuildInfo {
    /// Return the `NixBuildInfo` data ready for rendering the nix build file.
    pub fn for_config(config: &GenerateConfig) -> Result<BuildInfo, Error> {
        let metadata = cargo_metadata(config)?;
        let indexed_metadata = IndexedMetadata::new_from(metadata).map_err(|e| {
            format_err!(
                "while indexing metadata for {}: {}",
                config.cargo_toml.to_string_lossy(),
                e
            )
        })?;
        let mut default_nix = BuildInfo::new(config, indexed_metadata)?;

        prefetch_and_fill_crates_sha256(config, &mut default_nix)?;

        Ok(default_nix)
    }

    fn new(config: &GenerateConfig, metadata: IndexedMetadata) -> Result<BuildInfo, Error> {
        Ok(BuildInfo {
            root_package_id: metadata.root.clone(),
            crates: metadata
                .pkgs_by_id
                .values()
                .map(|package| CrateDerivation::resolve(config, &metadata, package))
                .collect::<Result<_, Error>>()?,
            indexed_metadata: metadata,
            info: GenerateInfo::new(),
            config: config.clone(),
        })
    }
}

/// Call `cargo metadata` and return result.
fn cargo_metadata(config: &GenerateConfig) -> Result<Metadata, Error> {
    let mut cmd = cargo_metadata::MetadataCommand::new();
    cmd.manifest_path(&config.cargo_toml);
    cmd.exec().map_err(|e| {
        format_err!(
            "while retrieving metadata about {}: {}",
            &config.cargo_toml.to_string_lossy(),
            e
        )
    })
}

/// Call `nix-prefetch` to determine sha256 for crates from crates.io.
fn prefetch_and_fill_crates_sha256(
    config: &GenerateConfig,
    default_nix: &mut BuildInfo,
) -> Result<(), Error> {
    let sha256_by_id: BTreeMap<PackageId, String> = crate::prefetch::prefetch_packages(
        config,
        default_nix.indexed_metadata.pkgs_by_id.values(),
    )
    .map_err(|e| format_err!("while prefetching crates for calculating sha256: {}", e))?;
    for some_crate in default_nix.crates.iter_mut() {
        some_crate.sha256 = sha256_by_id.get(&some_crate.package_id).cloned();
    }
    Ok(())
}

/// Some info about the crate2nix invocation.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GenerateInfo {
    pub crate2nix_version: String,
    pub crate2nix_arguments: Vec<String>,
}

impl GenerateInfo {
    fn new() -> GenerateInfo {
        GenerateInfo {
            crate2nix_version: env!("CARGO_PKG_VERSION").to_string(),
            crate2nix_arguments: env::args().skip(1).collect(),
        }
    }
}

/// Configuration for the default.nix generation.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GenerateConfig {
    pub cargo_toml: PathBuf,
    pub output: PathBuf,
    pub crate_hashes_json: PathBuf,
    pub nixpkgs_path: String,
}
