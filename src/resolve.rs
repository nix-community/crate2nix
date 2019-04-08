//! Resolve dependencies and other data for CrateDerivation.

use cargo_metadata::Dependency;
use cargo_metadata::DependencyKind;
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
use std::collections::HashSet;
use std::convert::Into;
use std::path::PathBuf;

use crate::metadata::IndexedMetadata;
use crate::target_cfg::{Cfg, CfgExpr};
use crate::GenerateConfig;

/// All data necessary for creating a derivation for a crate.
#[derive(Debug, Deserialize, Serialize)]
pub struct CrateDerivation {
    pub package_id: PackageId,
    pub crate_name: String,
    pub edition: String,
    pub authors: Vec<String>,
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
    // This derivation builds the root crate or a workspace member.
    pub is_root_or_workspace_member: bool,
}

impl CrateDerivation {
    pub fn resolve(
        config: &GenerateConfig,
        metadata: &IndexedMetadata,
        package: &Package,
    ) -> Result<CrateDerivation, Error> {
        let resolved_dependencies = ResolvedDependencies::new(metadata, package)?;

        let build_dependencies =
            resolved_dependencies.filtered_dependencies(|d| d.kind == DependencyKind::Build);
        let dependencies = resolved_dependencies.filtered_dependencies(|d| {
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
        let config_directory = config
            .cargo_toml
            .canonicalize()?
            .parent()
            .unwrap()
            .to_path_buf();

        let relative_source = if package_path == config_directory {
            "./.".into()
        } else {
            let path = diff_paths(package_path, &config_directory)
                .unwrap_or_else(|| package_path.to_path_buf());
            if path.starts_with("../") {
                path
            } else {
                PathBuf::from("./").join(path)
            }
        };

        let is_root_or_workspace_member = metadata.root.iter().chain(
            metadata
            .workspace_members
            .iter())
            .any(|pkg_id| *pkg_id == package.id);

        Ok(CrateDerivation {
            crate_name: package.name.clone(),
            edition: package.edition.clone(),
            authors: package.authors.clone(),
            package_id: package.id.clone(),
            version: package.version.clone(),
            // Will be filled later by prefetch_and_fill_crates_sha256.
            sha256: None,
            source_directory: relative_source,
            features: resolved_dependencies.node.features.clone(),
            dependencies,
            build_dependencies,
            build,
            lib_path,
            proc_macro,
            has_bin,
            is_root_or_workspace_member,
        })
    }
}

/// The resolved dependencies of one package/crate.
struct ResolvedDependencies<'a> {
    /// The node corresponding to the package.
    node: &'a Node,
    /// The corresponding packages for the dependencies.
    packages: Vec<&'a Package>,
    /// The dependencies filtered to the target platform.
    platform_dependencies: Vec<&'a Dependency>,
}

impl<'a> ResolvedDependencies<'a> {
    fn new(
        metadata: &'a IndexedMetadata,
        package: &'a Package,
    ) -> Result<ResolvedDependencies<'a>, Error> {
        let node: &Node = metadata.nodes_by_id.get(&package.id).ok_or_else(|| {
            format_err!(
                "Could not find node for {}.\n-- Package\n{}",
                &package.id,
                to_string_pretty(&package).unwrap_or_else(|_| "ERROR".to_string())
            )
        })?;

        let mut packages: Vec<&Package> =
            node
                .deps
                .iter()
                .map(|d| {
                    metadata.pkgs_by_id.get(&d.pkg).ok_or_else(|| {
                        format_err!(
                            "No matching package for dependency with package id {} in {}.\n-- Package\n{}\n-- Node\n{}",
                            d.pkg,
                            package.id,
                            to_string_pretty(&package).unwrap_or_else(|_| "ERROR".to_string()),
                            to_string_pretty(&node).unwrap_or_else(|_| "ERROR".to_string()),
                        )
                    })
                })
                .collect::<Result<_, Error>>()?;
        packages.sort_by(|p1, p2| p1.id.cmp(&p2.id));

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

        Ok(ResolvedDependencies {
            node,
            packages,
            platform_dependencies,
        })
    }

    fn filtered_dependencies(&self, filter: impl Fn(&Dependency) -> bool) -> Vec<PackageId> {
        /// Normalize a package name such as cargo does.
        fn normalize_package_name(package_name: &str) -> String {
            package_name.replace('-', "_")
        }

        let names: HashSet<String> = self
            .platform_dependencies
            .iter()
            .filter(|d| filter(**d))
            .map(|d| normalize_package_name(&d.name))
            .collect();
        self.packages
            .iter()
            .filter(|d| names.contains(&normalize_package_name(&d.name)))
            .map(|d| d.id.clone())
            .collect()
    }
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
