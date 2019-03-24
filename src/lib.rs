use std::collections::btree_map;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::env;
use std::fmt::Display;
use std::path::Path;
use std::path::PathBuf;

use cargo_metadata::Dependency;
use cargo_metadata::DependencyKind;
use cargo_metadata::Metadata;
use cargo_metadata::Node;
use cargo_metadata::Package;
use cargo_metadata::PackageId;
use failure::format_err;
use failure::Error;
use lazy_static::lazy_static;
use pathdiff::diff_paths;
use semver::Version;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::to_string_pretty;

use crate::target_cfg::Cfg;
use crate::target_cfg::CfgExpr;

mod target_cfg;
pub mod nix_build;
mod prefetch;
pub mod render;
mod util;

/// The input for the default.nix.tera template.
#[derive(Debug, Deserialize, Serialize)]
pub struct DefaultNix {
    pub root_derivation_name: Option<PackageId>,
    pub crates: Vec<Crate>,
    // For convenience include the source for tests.
    pub indexed_metadata: IndexedMetadata,
    // The generation configuration.
    pub info: GenerateInfo,
    // The generation configuration.
    pub config: GenerateConfig,
}

/// All data necessary for creating a build rule for a crate.
#[derive(Debug, Deserialize, Serialize)]
pub struct Crate {
    pub crate_name: String,
    pub edition: String,
    pub authors: Vec<String>,
    pub derivation_name: PackageId,
    pub version: Version,
    pub source_directory: PathBuf,
    pub sha256: Option<String>,
    pub dependencies: Vec<PackageId>,
    pub build_dependencies: Vec<PackageId>,
    pub features: Vec<String>,
    /// The relative path to the build script.
    pub build: Option<PathBuf>,
    pub lib_path: Option<PathBuf>,
    pub has_bin: bool,
    pub proc_macro: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GenerateInfo {
    pub cargo2nix_version: String,
    pub cargo2nix_arguments: Vec<String>,
}

impl GenerateInfo {
    fn new() -> GenerateInfo {
        GenerateInfo {
            cargo2nix_version: env!("CARGO_PKG_VERSION").to_string(),
            cargo2nix_arguments: env::args().skip(1).collect(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GenerateConfig {
    pub cargo_toml: PathBuf,
    pub crate_hashes_json: PathBuf,
    pub nixpkgs_path: String,
}

/// The metadata with maps indexed by {{PackageId}} instead of flat lists.
#[derive(Debug, Deserialize, Serialize)]
pub struct IndexedMetadata {
    pub root: Option<PackageId>,
    pub pkgs_by_id: BTreeMap<PackageId, Package>,
    pub nodes_by_id: BTreeMap<PackageId, Node>,
    pub sha256_by_id: BTreeMap<PackageId, String>,
}

impl IndexedMetadata {
    pub fn from(config: &GenerateConfig, metadata: Metadata) -> Result<IndexedMetadata, Error> {
        let resolve = metadata
            .resolve
            .ok_or_else(|| format_err!("no root in metadata"))?;

        let pkgs_by_id: BTreeMap<PackageId, Package> = metadata
            .packages
            .iter()
            .map(|pkg| (pkg.id.clone(), pkg.clone()))
            .collect();

        if pkgs_by_id.len() != metadata.packages.len() {
            let duplicate_ids =
                crate::util::find_duplicates(metadata.packages.iter().map(|p| &p.id));
            return Err(format_err!(
                "detected duplicate package IDs in metadata.packages: {:?}",
                duplicate_ids
            ));
        }

        let nodes_by_id: BTreeMap<PackageId, Node> = resolve
            .nodes
            .iter()
            .map(|node| (node.id.clone(), node.clone()))
            .collect();

        if nodes_by_id.len() != resolve.nodes.len() {
            let duplicate_ids = crate::util::find_duplicates(resolve.nodes.iter().map(|n| &n.id));
            return Err(format_err!(
                "detected duplicate package IDs in nodes: {:?}",
                duplicate_ids
            ));
        }

        let sha256_by_id: BTreeMap<PackageId, String> =
            crate::prefetch::prefetch_packages(config, &metadata.packages).map_err(|e| {
                format_err!("while prefetching crates for calculating sha256: {}", e)
            })?;

        Ok(IndexedMetadata {
            root: resolve.root,
            pkgs_by_id,
            nodes_by_id,
            sha256_by_id,
        })
    }
}

/// Normalize a package name such as cargo does.
fn normalize_package_name(package_name: &str) -> String {
    package_name.replace('-', "_")
}

impl Crate {
    fn from_package(
        config: &GenerateConfig,
        metadata: &IndexedMetadata,
        package: &Package,
    ) -> Result<Crate, Error> {
        let node = metadata.nodes_by_id.get(&package.id).ok_or_else(|| {
            format_err!(
                "Could not find node for {}.\n-- Package\n{}",
                &package.id,
                to_string_pretty(&package).unwrap_or("ERROR".to_string())
            )
        })?;

        let dependency_packages: Vec<&Package> =
            node
                .deps
                .iter()
                .map(|d| {
                    metadata.pkgs_by_id.get(&d.pkg).ok_or_else(|| {
                        format_err!(
                            "No matching package for dependency with package id {} in {}.\n-- Package\n{}\n-- Node\n{}",
                            d.pkg,
                            package.id,
                            to_string_pretty(&package).unwrap_or("ERROR".to_string()),
                            to_string_pretty(&node).unwrap_or("ERROR".to_string()),
                        )
                    })
                })
                .collect::<Result<_, Error>>()?;

        let platform_dependencies: Vec<&Dependency> = package
            .dependencies
            .iter()
            .filter(|d| {
                d.target
                    .as_ref()
                    .map(|platform| CfgExpr::matches_key(&platform.to_string(), &PLATFORM))
                    .unwrap_or(true)
            })
            .collect();

        fn filter_dependencies(
            dependency_packages: &Vec<&Package>,
            dependencies: &Vec<&Dependency>,
            filter: impl Fn(&&&Dependency) -> bool,
        ) -> Vec<PackageId> {
            let names: HashSet<String> = dependencies
                .iter()
                .filter(filter)
                .map(|d| normalize_package_name(&d.name))
                .collect();
            dependency_packages
                .iter()
                .filter(|d| names.contains(&normalize_package_name(&d.name)))
                .map(|d| d.id.clone())
                .collect()
        }

        let build_dependencies =
            filter_dependencies(&dependency_packages, &platform_dependencies, |d| {
                d.kind == DependencyKind::Build
            });
        let dependencies = filter_dependencies(&dependency_packages, &platform_dependencies, |d| {
            d.kind == DependencyKind::Normal || d.kind == DependencyKind::Unknown
        });

        let package_path = package
            .manifest_path
            .parent()
            .expect("WUUT? No parent directory of manifest?");

        let lib_path = package
            .targets
            .iter()
            .find(|t| t.kind.iter().any(|k| k == "lib"))
            .and_then(|target| target.src_path.strip_prefix(package_path).ok())
            .map(|path| path.to_path_buf());

        let build = package
            .targets
            .iter()
            .find(|t| t.kind.iter().any(|k| k == "custom-build"))
            .and_then(|target| target.src_path.strip_prefix(package_path).ok())
            .map(|path| path.to_path_buf());

        let proc_macro = package
            .targets
            .iter()
            .any(|t| t.kind.iter().any(|k| k == "proc-macro"));

        let has_bin = package
            .targets
            .iter()
            .any(|t| t.kind.iter().any(|k| k == "bin"));
        let config_directory = config.cargo_toml.canonicalize()?.parent().unwrap().to_path_buf();

        let relative_source = if package_path == config_directory {
            "./.".into()
        } else {
            diff_paths(package_path, &config_directory)
                .unwrap_or(package_path.to_path_buf())
        };

        Ok(Crate {
            crate_name: package.name.clone(),
            edition: package.edition.clone(),
            authors: package.authors.clone(),
            derivation_name: package.id.clone(),
            version: package.version.clone(),
            sha256: metadata.sha256_by_id.get(&package.id).map(|s| s.clone()),
            source_directory: relative_source,
            features: node.features.clone(),
            dependencies,
            build_dependencies,
            build,
            lib_path,
            proc_macro,
            has_bin,
        })
    }
}

impl DefaultNix {
    fn new(config: &GenerateConfig, metadata: IndexedMetadata) -> Result<DefaultNix, Error> {
        Ok(DefaultNix {
            root_derivation_name: metadata.root.clone(),
            crates: metadata
                .pkgs_by_id
                .values()
                .map(|package| Crate::from_package(config, &metadata, package))
                .collect::<Result<_, Error>>()?,
            indexed_metadata: metadata,
            info: GenerateInfo::new(),
            config: config.clone(),
        })
    }
}

pub fn cargo_metadata(config: &GenerateConfig) -> Result<IndexedMetadata, Error> {
    let mut cmd = cargo_metadata::MetadataCommand::new();
    cmd.manifest_path(&config.cargo_toml);

    fn err_for_cargo_toml(cargo_toml: &Path, e: impl Display) -> Error {
        format_err!(
            "while retrieving metadata about {}: {}",
            cargo_toml.to_string_lossy(),
            e
        )
    }

    cmd.exec()
        .map_err(|e| err_for_cargo_toml(&config.cargo_toml, e))
        .and_then(|m| IndexedMetadata::from(config, m))
        .map_err(|e| err_for_cargo_toml(&config.cargo_toml, e))
}

pub fn default_nix(config: &GenerateConfig) -> Result<DefaultNix, Error> {
    let indexed_metadata = cargo_metadata(config)?;
    DefaultNix::new(config, indexed_metadata)
}

lazy_static! {
    /// The platform configuration that we will evaluate dependency target expressions against.
    ///
    /// This basically means that it only works if the target is Linux x86_64.
    ///
    /// TODO(pkolloch): Remove this restriction
    static ref PLATFORM: Vec<Cfg> = {
        fn name(n: &str) -> Cfg {
            Cfg::Name(n.to_string())
        }
        fn key_pair(k: &str, v: &str) -> Cfg {
            Cfg::KeyPair(k.to_string(), v.to_string())
        }

        vec![
            name("debug_assertions"),
            name("unix"),
            key_pair("target_endian", "little"),
            key_pair("target_env", "gnu"),
            key_pair("target_family", "unix"),
            key_pair("target_feature", "fxsr"),
            key_pair("target_feature", "sse"),
            key_pair("target_feature", "sse2"),
            key_pair("target_os", "linux"),
            key_pair("target_pointer_width", "64"),
            key_pair("target_vendor", "unknown"),
        ]
    };
}
