//! Resolve dependencies and other data for CrateDerivation.

use cargo_metadata::DependencyKind;
use cargo_metadata::Node;
use cargo_metadata::Package;
use cargo_metadata::PackageId;
use cargo_metadata::{Dependency, Source};
use failure::format_err;
use failure::Error;
use pathdiff::diff_paths;
use semver::Version;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::to_string_pretty;
use std::collections::HashMap;
use std::convert::Into;
use std::path::{Path, PathBuf};

use crate::metadata::IndexedMetadata;
use crate::GenerateConfig;
use url::Url;

/// All data necessary for creating a derivation for a crate.
#[derive(Debug, Deserialize, Serialize)]
pub struct CrateDerivation {
    pub package_id: PackageId,
    pub crate_name: String,
    pub edition: String,
    pub authors: Vec<String>,
    pub version: Version,
    pub source: ResolvedSource,
    pub dependencies: Vec<ResolvedDependency>,
    pub build_dependencies: Vec<ResolvedDependency>,
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
            .expect("WUUT? No parent directory of manifest?")
            .canonicalize()
            .expect("Cannot canonicalize package path");

        let lib_path = package
            .targets
            .iter()
            .find(|t| t.kind.iter().any(|k| k == "lib"))
            .and_then(|target| target.src_path.strip_prefix(&package_path).ok())
            .map(|path| path.to_path_buf());

        let build = package
            .targets
            .iter()
            .find(|t| t.kind.iter().any(|k| k == "custom-build"))
            .and_then(|target| target.src_path.strip_prefix(&package_path).ok())
            .map(|path| path.to_path_buf());

        let proc_macro = package
            .targets
            .iter()
            .any(|t| t.kind.iter().any(|k| k == "proc-macro"));

        let has_bin = package
            .targets
            .iter()
            .any(|t| t.kind.iter().any(|k| k == "bin"));

        let is_root_or_workspace_member = metadata
            .root
            .iter()
            .chain(metadata.workspace_members.iter())
            .any(|pkg_id| *pkg_id == package.id);

        Ok(CrateDerivation {
            crate_name: package.name.clone(),
            edition: package.edition.clone(),
            authors: package.authors.clone(),
            package_id: package.id.clone(),
            version: package.version.clone(),
            source: ResolvedSource::new(&config, &package, &package_path)?,
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

/// Specifies how to retrieve the source code.
#[derive(Debug, Deserialize, Serialize)]
pub enum ResolvedSource {
    CratesIo {
        sha256: Option<String>,
    },
    Git {
        #[serde(with = "url_serde")]
        url: Url,
        rev: String,
    },
    LocalDirectory {
        path: PathBuf,
    },
}

const GIT_SOURCE_PREFIX: &str = "git+";

impl ResolvedSource {
    pub fn new(
        config: &GenerateConfig,
        package: &Package,
        package_path: impl AsRef<Path>,
    ) -> Result<ResolvedSource, Error> {
        match package.source.as_ref() {
            Some(source) if source.is_crates_io() => {
                // Will sha256 will be filled later by prefetch_and_fill_crates_sha256.
                Ok(ResolvedSource::CratesIo { sha256: None })
            }
            Some(source) => {
                ResolvedSource::git_or_local_directory(config, package, &package_path, source)
            }
            None => Ok(ResolvedSource::LocalDirectory {
                path: ResolvedSource::relative_directory(config, package_path)?,
            }),
        }
    }

    fn git_or_local_directory(
        config: &GenerateConfig,
        package: &Package,
        package_path: &impl AsRef<Path>,
        source: &Source,
    ) -> Result<ResolvedSource, Error> {
        let source_string = source.to_string();
        if !source_string.starts_with(GIT_SOURCE_PREFIX) {
            return ResolvedSource::fallback_to_local_directory(
                config,
                package,
                package_path,
                "No 'git+' prefix found.",
            );
        }
        let mut url = url::Url::parse(&source_string[GIT_SOURCE_PREFIX.len()..])?;
        let rev = if let Some((_, rev)) = url.query_pairs().find(|(k, _)| k == "rev") {
            rev.to_string()
        } else if let Some(rev) = url.fragment() {
            rev.to_string()
        } else {
            return ResolvedSource::fallback_to_local_directory(
                config,
                package,
                package_path,
                "No git revision found.",
            );
        };
        url.set_query(None);
        url.set_fragment(None);
        Ok(ResolvedSource::Git { url, rev })
    }

    fn fallback_to_local_directory(
        config: &GenerateConfig,
        package: &Package,
        package_path: impl AsRef<Path>,
        warning: &str,
    ) -> Result<ResolvedSource, Error> {
        let path = Self::relative_directory(config, package_path)?;
        eprintln!(
            "WARNING: {} Falling back to local directory for crate {} with source {}: {}",
            warning,
            package.id,
            package
                .source
                .as_ref()
                .map(|s| s.to_string())
                .unwrap_or("N/A".to_string()),
            &path.to_string_lossy()
        );
        return Ok(ResolvedSource::LocalDirectory { path });
    }

    fn relative_directory(
        config: &GenerateConfig,
        package_path: impl AsRef<Path>,
    ) -> Result<PathBuf, Error> {
        // Use local directory. This is the local cargo crate directory in the worst case.

        let mut output_build_file_directory = config.output.parent().ok_or_else(|| {
            format_err!(
                "could not get parent of output file '{}'.",
                config.output.to_string_lossy()
            )
        })?.to_path_buf();

        if output_build_file_directory.is_relative() {
            // Deal with "empty" path. E.g. the parent of "Cargo.nix" is "".
            output_build_file_directory = Path::new(".").join(output_build_file_directory);
        }

        output_build_file_directory = output_build_file_directory.canonicalize().map_err(|e| {
            format_err!(
                "could not canonicalize output file directory '{}': {}",
                output_build_file_directory.to_string_lossy(), e
            )
        })?;

        Ok(if package_path.as_ref() == output_build_file_directory {
            "./.".into()
        } else {
            let path = diff_paths(package_path.as_ref(), &output_build_file_directory)
                .unwrap_or_else(|| package_path.as_ref().to_path_buf());
            if path.starts_with("../") {
                path
            } else {
                PathBuf::from("./").join(path)
            }
        })
    }
}

/// The resolved dependencies of one package/crate.
struct ResolvedDependencies<'a> {
    /// The node corresponding to the package.
    node: &'a Node,
    /// The corresponding packages for the dependencies.
    packages: Vec<&'a Package>,
    /// The dependencies of the package/crate.
    dependencies: Vec<&'a Dependency>,
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

        Ok(ResolvedDependencies {
            node,
            packages,
            dependencies: package.dependencies.iter().collect(),
        })
    }

    fn filtered_dependencies(
        &self,
        filter: impl Fn(&Dependency) -> bool,
    ) -> Vec<ResolvedDependency> {
        /// Normalize a package name such as cargo does.
        fn normalize_package_name(package_name: &str) -> String {
            package_name.replace('-', "_")
        }

        let names: HashMap<String, &&Dependency> = self
            .dependencies
            .iter()
            .filter(|d| filter(**d))
            .map(|d| (normalize_package_name(&d.name), d))
            .collect();
        self.packages
            .iter()
            .flat_map(|d| {
                names
                    .get(&normalize_package_name(&d.name))
                    .map(|dependency| ResolvedDependency {
                        package_id: d.id.clone(),
                        target: dependency.target.as_ref().map(|p| p.to_string()),
                    })
            })
            .collect()
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResolvedDependency {
    pub package_id: PackageId,
    /// The cfg expression for conditionally enabling the dependency (if any).
    /// Can also be a target "triplet".
    pub target: Option<String>,
}
