//! Resolve dependencies and other data for CrateDerivation.

use anyhow::format_err;
use anyhow::Error;
use cargo_metadata::Node;
use cargo_metadata::Package;
use cargo_metadata::PackageId;
use cargo_metadata::{Dependency, Source};
use cargo_metadata::{DependencyKind, Target};
use cargo_platform::Platform;
use pathdiff::diff_paths;
use semver::Version;
use serde::Deserialize;
use serde::Serialize;
use serde_json::to_string_pretty;
use std::collections::HashMap;
use std::convert::Into;
use std::path::{Path, PathBuf};

use crate::metadata::IndexedMetadata;
#[cfg(test)]
use crate::test;
use crate::GenerateConfig;
use itertools::Itertools;
use std::{collections::btree_map::BTreeMap, fmt::Display};
use url::Url;

/// All data necessary for creating a derivation for a crate.
#[derive(Debug, Deserialize, Serialize)]
pub struct CrateDerivation {
    pub package_id: PackageId,
    pub crate_name: String,
    pub edition: String,
    pub authors: Vec<String>,
    pub version: Version,
    /// The name of a native library the package is linking to.
    pub links: Option<String>,
    pub source: ResolvedSource,
    /// The crate types of the lib targets of this crate, e.g. "lib", "dylib", "rlib", ...
    pub lib_crate_types: Vec<String>,
    pub dependencies: Vec<ResolvedDependency>,
    pub build_dependencies: Vec<ResolvedDependency>,
    pub dev_dependencies: Vec<ResolvedDependency>,
    /// Feature rules. Which feature (key) enables which other features (values).
    pub features: BTreeMap<String, Vec<String>>,
    /// The resolved features for this crate for a default build as returned by cargo.
    pub resolved_default_features: Vec<String>,
    /// The build target for the custom build script.
    pub build: Option<BuildTarget>,
    /// The build target for the library.
    pub lib: Option<BuildTarget>,
    pub binaries: Vec<BuildTarget>,
    pub proc_macro: bool,
    /// This derivation builds the root crate or a workspace member.
    pub is_root_or_workspace_member: bool,
}

impl CrateDerivation {
    pub fn resolve(
        config: &GenerateConfig,
        crate2nix_json: &crate::config::Config,
        metadata: &IndexedMetadata,
        package: &Package,
    ) -> Result<CrateDerivation, Error> {
        let resolved_dependencies = ResolvedDependencies::new(metadata, package)?;

        let build_dependencies =
            resolved_dependencies.filtered_dependencies(|d| d.kind == DependencyKind::Build);

        let dependencies = resolved_dependencies.filtered_dependencies(|d| {
            d.kind == DependencyKind::Normal || d.kind == DependencyKind::Unknown
        });

        let dev_dependencies =
            resolved_dependencies.filtered_dependencies(|d| d.kind == DependencyKind::Development);

        let is_root_or_workspace_member = metadata
            .root
            .iter()
            .chain(metadata.workspace_members.iter())
            .any(|pkg_id| *pkg_id == package.id);

        let package_path = package.manifest_path.parent().unwrap_or_else(|| {
            panic!(
                "WUUT? No parent directory of manifest at {}?",
                package.manifest_path.as_str()
            )
        });

        // This depends on the non-cananocalized package_path (without symlinks
        // resolved).
        let configured_source = if is_root_or_workspace_member {
            // In the resolved data, we don't have the link to the workspace member
            // name anymore. So we need to extract it from the path.
            let configured_source = package_path
                .file_name()
                .and_then(|file_name| crate2nix_json.sources.get(file_name).cloned());

            if !crate2nix_json.sources.is_empty() && configured_source.is_none() {
                eprintln!(
                    "warning: Could not find configured source for workspace member {:?}",
                    package_path
                );
            }

            configured_source
        } else {
            None
        };

        let source = if let Some(configured) = configured_source {
            configured.into()
        } else {
            ResolvedSource::new(config, package, package_path)?
        };

        let package_path = package_path.canonicalize().map_err(|e| {
            format_err!(
                "while canonicalizing crate path path {}: {}",
                package_path.as_str(),
                e
            )
        })?;

        let lib = package
            .targets
            .iter()
            .find(|t| {
                t.kind.iter().any(|k| {
                    k == "lib" || k == "cdylib" || k == "dylib" || k == "rlib" || k == "proc-macro"
                })
            })
            .and_then(|target| BuildTarget::new(target, &package_path).ok());

        let build = package
            .targets
            .iter()
            .find(|t| t.kind.iter().any(|k| k == "custom-build"))
            .and_then(|target| BuildTarget::new(target, &package_path).ok());

        let proc_macro = package
            .targets
            .iter()
            .any(|t| t.kind.iter().any(|k| k == "proc-macro"));

        let binaries = package
            .targets
            .iter()
            .filter_map(|t| {
                if t.kind.iter().any(|k| k == "bin") {
                    BuildTarget::new(t, &package_path).ok()
                } else {
                    None
                }
            })
            .collect();

        Ok(CrateDerivation {
            crate_name: package.name.clone(),
            edition: package.edition.to_string(),
            authors: package.authors.clone(),
            package_id: package.id.clone(),
            version: package.version.clone(),
            links: package.links.clone(),
            source,
            features: package
                .features
                .iter()
                .map(|(name, feature_list)| (name.clone(), feature_list.clone()))
                .collect(),
            resolved_default_features: metadata
                .nodes_by_id
                .get(&package.id)
                .map(|n| n.features.clone())
                .unwrap_or_default(),
            lib_crate_types: package
                .targets
                .iter()
                .filter(|target| target.kind.iter().any(|kind| kind.ends_with("lib")))
                .flat_map(|target| target.crate_types.iter())
                .unique()
                .cloned()
                .collect(),
            dependencies,
            build_dependencies,
            dev_dependencies,
            build,
            lib,
            proc_macro,
            binaries,
            is_root_or_workspace_member,
        })
    }
}

#[test]
pub fn minimal_resolve() {
    use cargo_metadata::{Metadata, Resolve};

    let config = test::generate_config();

    let package = test::package("main", "1.2.3");
    let node = test::node(&package.id.repr);

    let mut resolve: Resolve = test::empty_resolve();
    resolve.root = Some(package.id.clone());
    resolve.nodes = vec![node];

    let mut metadata: Metadata = test::empty_metadata();
    metadata.workspace_members = vec![package.id.clone()];
    metadata.packages = vec![package.clone()];
    metadata.resolve = Some(resolve);

    let indexed = IndexedMetadata::new_from(metadata).unwrap();

    println!("indexed: {:#?}", indexed);

    let root_package = &indexed.root_package().expect("root package");
    let crate_derivation = CrateDerivation::resolve(
        &config,
        &crate::config::Config::default(),
        &indexed,
        root_package,
    )
    .unwrap();

    println!("crate_derivation: {:#?}", crate_derivation);

    assert_eq!(crate_derivation.crate_name, "main");
    assert_eq!(
        crate_derivation.version,
        semver::Version::parse("1.2.3").unwrap()
    );
    assert!(crate_derivation.is_root_or_workspace_member);
    let empty: Vec<String> = vec![];
    assert_eq!(crate_derivation.lib_crate_types, empty);

    package.close().unwrap();
}

#[test]
pub fn configured_source_is_used_instead_of_local_directory() {
    use std::convert::TryInto;
    use std::str::FromStr;

    let mut env = test::MetadataEnv::default();
    let config = test::generate_config();

    // crate2nix creates a "virtual" workspace which consists of symlinks to the member sources.
    // The symlinks use the source names and this is how we detect that we use a workspace member
    // source.
    // By simulating this layout, we ensure that we do not canonicalize paths at the "wrong"
    // moment.
    let simulated_store_path = env.temp_dir();
    std::fs::File::create(simulated_store_path.join("Cargo.toml")).expect("File creation failed");
    let workspace_with_symlink = env.temp_dir();
    std::os::unix::fs::symlink(
        &simulated_store_path,
        workspace_with_symlink.join("some_crate"),
    )
    .expect("could not create symlink");
    let manifest_path = workspace_with_symlink.join("some_crate").join("Cargo.toml");

    let mut main = env.add_package_and_node("main");
    main.update_package(|p| p.manifest_path = manifest_path.try_into().unwrap());
    main.make_root();

    let indexed = env.indexed_metadata();

    let root_package = &indexed.root_package().expect("root package");

    let mut crate2nix_json = crate::config::Config::default();
    let source = crate::config::Source::CratesIo {
        name: "some_crate".to_string(),
        version: semver::Version::from_str("1.2.3").unwrap(),
        sha256: "123".to_string(),
    };
    crate2nix_json.upsert_source(None, source);
    let crate_derivation =
        CrateDerivation::resolve(&config, &crate2nix_json, &indexed, root_package).unwrap();

    println!("crate_derivation: {:#?}", crate_derivation);

    assert!(crate_derivation.is_root_or_workspace_member);
    assert_eq!(
        crate_derivation.source,
        ResolvedSource::CratesIo(CratesIoSource {
            name: "some_crate".to_string(),
            version: semver::Version::from_str("1.2.3").unwrap(),
            sha256: Some("123".to_string()),
        })
    );

    env.close();
}

#[test]
pub fn double_crate_with_rename() {
    let mut env = test::MetadataEnv::default();
    let config = test::generate_config();

    let mut main = env.add_package_and_node("main");
    main.make_root();
    main.add_dependency("futures")
        .version_and_package_id("0.1.0")
        .update_package_dep(|d| d.rename = Some("futures01".to_string()))
        .update_node_dep(|n| n.name = "futures01".to_string());
    main.add_dependency("futures")
        .version_and_package_id("0.3.0")
        .update_package_dep(|d| {
            d.uses_default_features = false;
            d.features = vec!["compat".to_string()];
        });

    let indexed = env.indexed_metadata();

    let root_package = &indexed.root_package().expect("root package");

    let crate_derivation = CrateDerivation::resolve(
        &config,
        &crate::config::Config::default(),
        &indexed,
        root_package,
    )
    .unwrap();

    println!("crate_derivation: {:#?}", crate_derivation);

    assert_eq!(crate_derivation.dependencies.len(), 2);

    env.close();
}

/// A build target of a crate.
#[derive(Debug, Deserialize, Serialize)]
pub struct BuildTarget {
    /// The name of the build target.
    pub name: String,
    /// The relative path of the target source file.
    pub src_path: PathBuf,
    /// The crate's features that need to be enabled for this target to be compiled.
    /// Otherwise, this target is ignored.
    pub required_features: Vec<String>,
}

impl BuildTarget {
    pub fn new(target: &Target, package_path: impl AsRef<Path>) -> Result<BuildTarget, Error> {
        Ok(BuildTarget {
            name: target.name.clone(),
            src_path: target
                .src_path
                .canonicalize()?
                .strip_prefix(&package_path)?
                .to_path_buf(),
            required_features: target.required_features.clone(),
        })
    }
}

/// Specifies how to retrieve the source code.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum ResolvedSource {
    CratesIo(CratesIoSource),
    Registry(RegistrySource),
    Git(GitSource),
    LocalDirectory(LocalDirectorySource),
    Nix(NixSource),
}

impl From<crate::config::Source> for ResolvedSource {
    fn from(source: crate::config::Source) -> Self {
        match source {
            crate::config::Source::Git { url, rev, sha256 } => ResolvedSource::Git(GitSource {
                url,
                rev,
                r#ref: None,
                sha256: Some(sha256),
                resolved_cargo_toml: None,
            }),
            crate::config::Source::CratesIo {
                name,
                version,
                sha256,
            } => ResolvedSource::CratesIo(CratesIoSource {
                name,
                version,
                sha256: Some(sha256),
            }),
            crate::config::Source::Registry {
                name,
                version,
                sha256,
                registry,
            } => ResolvedSource::Registry(RegistrySource {
                name,
                version,
                sha256: Some(sha256),
                registry: registry.parse().unwrap(),
            }),
            crate::config::Source::Nix { file, attr } => {
                ResolvedSource::Nix(NixSource { file, attr })
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct CratesIoSource {
    pub name: String,
    pub version: Version,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct RegistrySource {
    pub registry: Url,
    pub name: String,
    pub version: Version,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct GitSource {
    pub url: Url,
    pub rev: String,
    pub r#ref: Option<String>,
    pub sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_cargo_toml: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct LocalDirectorySource {
    path: PathBuf,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct NixSource {
    file: crate::config::NixFile,
    attr: Option<String>,
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
                Ok(ResolvedSource::CratesIo(CratesIoSource {
                    name: package.name.clone(),
                    version: package.version.clone(),
                    sha256: None,
                }))
            }
            Some(source) if source.repr.starts_with("sparse+") => {
                Ok(ResolvedSource::Registry(RegistrySource {
                    registry: source
                        .repr
                        .split_at("sparse+".len())
                        .1
                        .to_string()
                        .parse()
                        .unwrap(),
                    name: package.name.clone(),
                    version: package.version.clone(),
                    sha256: None,
                }))
            }
            Some(source) => {
                ResolvedSource::git_or_local_directory(config, package, &package_path, source)
            }
            None => Ok(ResolvedSource::LocalDirectory(LocalDirectorySource {
                path: ResolvedSource::relative_directory(config, package_path)?,
            })),
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
        let mut query_pairs = url.query_pairs();
        let branch = query_pairs
            .find(|(k, _)| k == "branch")
            .map(|(_, v)| v.to_string());
        let rev = if let Some((_, rev)) = query_pairs.find(|(k, _)| k == "rev") {
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

        let resolved_cargo_toml = {
            let manifest = package.manifest_path.as_std_path();
            let content = std::fs::read_to_string(manifest).ok();
            match content {
                Some(c) if c.contains("workspace = true") => Some(reconstruct_cargo_toml(package)),
                _ => None,
            }
        };

        Ok(ResolvedSource::Git(GitSource {
            url,
            rev,
            r#ref: branch,
            sha256: None,
            resolved_cargo_toml,
        }))
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
                .map(std::string::ToString::to_string)
                .unwrap_or_else(|| "N/A".to_string()),
            &path.to_string_lossy()
        );
        Ok(ResolvedSource::LocalDirectory(LocalDirectorySource {
            path,
        }))
    }

    fn relative_directory(
        config: &GenerateConfig,
        package_path: impl AsRef<Path>,
    ) -> Result<PathBuf, Error> {
        // Use local directory. This is the local cargo crate directory in the worst case.

        let mut output_build_file_directory = config
            .output
            .parent()
            .ok_or_else(|| {
                format_err!(
                    "could not get parent of output file '{}'.",
                    config.output.to_string_lossy()
                )
            })?
            .to_path_buf();

        if output_build_file_directory.is_relative() {
            // Deal with "empty" path. E.g. the parent of "Cargo.nix" is "".
            output_build_file_directory = Path::new(".").join(output_build_file_directory);
        }

        output_build_file_directory = output_build_file_directory.canonicalize().map_err(|e| {
            format_err!(
                "could not canonicalize output file directory '{}': {}",
                output_build_file_directory.to_string_lossy(),
                e
            )
        })?;

        Ok(if package_path.as_ref() == output_build_file_directory {
            "./.".into()
        } else {
            let path = diff_paths(package_path.as_ref(), &output_build_file_directory)
                .unwrap_or_else(|| package_path.as_ref().to_path_buf());
            if path == PathBuf::from("../") {
                path.join(PathBuf::from("."))
            } else if path.starts_with("../") {
                path
            } else {
                PathBuf::from("./").join(path)
            }
        })
    }

    pub fn sha256(&self) -> Option<&String> {
        match self {
            Self::CratesIo(CratesIoSource { sha256, .. })
            | Self::Registry(RegistrySource { sha256, .. })
            | Self::Git(GitSource { sha256, .. }) => sha256.as_ref(),
            _ => None,
        }
    }

    pub fn with_sha256(&self, sha256: String) -> Self {
        match self {
            Self::CratesIo(source) => Self::CratesIo(CratesIoSource {
                sha256: Some(sha256),
                ..source.clone()
            }),
            Self::Registry(source) => Self::Registry(RegistrySource {
                sha256: Some(sha256),
                ..source.clone()
            }),
            Self::Git(source) => Self::Git(GitSource {
                sha256: Some(sha256),
                ..source.clone()
            }),
            _ => self.clone(),
        }
    }

    pub fn without_sha256(&self) -> Self {
        match self {
            Self::CratesIo(source) => Self::CratesIo(CratesIoSource {
                sha256: None,
                ..source.clone()
            }),
            Self::Registry(source) => Self::Registry(RegistrySource {
                sha256: None,
                ..source.clone()
            }),
            Self::Git(source) => Self::Git(GitSource {
                sha256: None,
                ..source.clone()
            }),
            _ => self.clone(),
        }
    }
}

impl ToString for ResolvedSource {
    fn to_string(&self) -> String {
        match self {
            Self::CratesIo(source) => source.to_string(),
            Self::Registry(source) => source.to_string(),
            Self::Git(source) => source.to_string(),
            Self::LocalDirectory(source) => source.to_string(),
            Self::Nix(source) => source.to_string(),
        }
    }
}

impl CratesIoSource {
    pub fn url(&self) -> String {
        // https://www.pietroalbini.org/blog/downloading-crates-io/
        // Not rate-limited, CDN URL.
        format!(
            "https://static.crates.io/crates/{name}/{name}-{version}.crate",
            name = self.name,
            version = self.version
        )
    }
}

impl RegistrySource {
    pub fn url(&self) -> String {
        unimplemented!()
    }
}

impl Display for CratesIoSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url())
    }
}

impl Display for RegistrySource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url())
    }
}

impl Display for GitSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let base = format!("{}#{}", self.url, self.rev);
        if let Some(branch) = self.r#ref.as_ref() {
            write!(f, "{} branch: {}", base, branch)
        } else {
            write!(f, "{}", base)
        }
    }
}

impl Display for LocalDirectorySource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.to_str().unwrap())
    }
}

impl Display for NixSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(attr) = self.attr.as_ref() {
            write!(f, "({}).{}", self.file, attr)
        } else {
            write!(f, "{}", self.file)
        }
    }
}

/// Reconstruct a standalone Cargo.toml from a `cargo_metadata::Package`.
///
/// When cargo resolves workspace inheritance, the `Package` struct contains
/// all resolved values. This function serializes those back into a valid
/// standalone Cargo.toml that doesn't reference any workspace.
fn reconstruct_cargo_toml(package: &Package) -> String {
    use toml::Value;

    type TomlMap = toml::map::Map<String, toml::Value>;

    let mut doc = toml::map::Map::new();

    // [package] section
    let mut pkg = toml::map::Map::new();
    pkg.insert("name".into(), Value::String(package.name.clone()));
    pkg.insert("version".into(), Value::String(package.version.to_string()));
    pkg.insert("edition".into(), Value::String(package.edition.to_string()));
    if !package.authors.is_empty() {
        pkg.insert(
            "authors".into(),
            Value::Array(
                package
                    .authors
                    .iter()
                    .map(|a| Value::String(a.clone()))
                    .collect(),
            ),
        );
    }
    if let Some(ref description) = package.description {
        pkg.insert("description".into(), Value::String(description.clone()));
    }
    if let Some(ref license) = package.license {
        pkg.insert("license".into(), Value::String(license.clone()));
    }
    if let Some(ref links) = package.links {
        pkg.insert("links".into(), Value::String(links.clone()));
    }
    if let Some(ref repository) = package.repository {
        pkg.insert("repository".into(), Value::String(repository.clone()));
    }
    if let Some(ref homepage) = package.homepage {
        pkg.insert("homepage".into(), Value::String(homepage.clone()));
    }
    if let Some(ref documentation) = package.documentation {
        pkg.insert("documentation".into(), Value::String(documentation.clone()));
    }
    if let Some(ref readme) = package.readme {
        pkg.insert("readme".into(), Value::String(readme.to_string()));
    }
    if let Some(ref rust_version) = package.rust_version {
        pkg.insert(
            "rust-version".into(),
            Value::String(rust_version.to_string()),
        );
    }
    if let Some(ref default_run) = package.default_run {
        pkg.insert("default-run".into(), Value::String(default_run.clone()));
    }
    if let Some(ref publish) = package.publish {
        pkg.insert(
            "publish".into(),
            Value::Array(publish.iter().map(|s| Value::String(s.clone())).collect()),
        );
    }
    doc.insert("package".into(), Value::Table(pkg));

    // [lib] target
    for target in &package.targets {
        if target.kind.iter().any(|k| {
            k == "lib" || k == "rlib" || k == "dylib" || k == "cdylib" || k == "proc-macro"
        }) {
            let mut lib = toml::map::Map::new();
            lib.insert("name".into(), Value::String(target.name.clone()));
            if let Some(rel_path) = relative_target_path(package, target) {
                lib.insert("path".into(), Value::String(rel_path));
            }
            if target.kind.iter().any(|k| k == "proc-macro") {
                lib.insert("proc-macro".into(), Value::Boolean(true));
            }
            let non_default_types: Vec<_> = target
                .crate_types
                .iter()
                .filter(|ct| *ct != "lib")
                .cloned()
                .collect();
            if !non_default_types.is_empty() {
                lib.insert(
                    "crate-type".into(),
                    Value::Array(
                        target
                            .crate_types
                            .iter()
                            .map(|s| Value::String(s.clone()))
                            .collect(),
                    ),
                );
            }
            doc.insert("lib".into(), Value::Table(lib));
        }
    }

    // [[bin]] targets
    let bins: Vec<_> = package
        .targets
        .iter()
        .filter(|t| t.kind.iter().any(|k| k == "bin"))
        .collect();
    if !bins.is_empty() {
        let bin_array: Vec<Value> = bins
            .iter()
            .map(|target| {
                let mut bin = toml::map::Map::new();
                bin.insert("name".into(), Value::String(target.name.clone()));
                if let Some(rel_path) = relative_target_path(package, target) {
                    bin.insert("path".into(), Value::String(rel_path));
                }
                if !target.required_features.is_empty() {
                    bin.insert(
                        "required-features".into(),
                        Value::Array(
                            target
                                .required_features
                                .iter()
                                .map(|f| Value::String(f.clone()))
                                .collect(),
                        ),
                    );
                }
                Value::Table(bin)
            })
            .collect();
        doc.insert("bin".into(), Value::Array(bin_array));
    }

    // Dependencies grouped by kind and target
    let mut deps = toml::map::Map::new();
    let mut dev_deps = toml::map::Map::new();
    let mut build_deps = toml::map::Map::new();
    let mut target_deps: BTreeMap<String, (TomlMap, TomlMap, TomlMap)> = BTreeMap::new();

    for dep in &package.dependencies {
        let dep_value = dependency_to_toml(dep);
        let target_key = dep.target.as_ref().map(|t| t.to_string());

        match (dep.kind, target_key) {
            (DependencyKind::Normal | DependencyKind::Unknown, None) => {
                deps.insert(dep.name.clone(), dep_value);
            }
            (DependencyKind::Development, None) => {
                dev_deps.insert(dep.name.clone(), dep_value);
            }
            (DependencyKind::Build, None) => {
                build_deps.insert(dep.name.clone(), dep_value);
            }
            (kind, Some(target_str)) => {
                let entry = target_deps
                    .entry(target_str)
                    .or_insert_with(|| (TomlMap::new(), TomlMap::new(), TomlMap::new()));
                match kind {
                    DependencyKind::Normal | DependencyKind::Unknown => {
                        entry.0.insert(dep.name.clone(), dep_value);
                    }
                    DependencyKind::Development => {
                        entry.1.insert(dep.name.clone(), dep_value);
                    }
                    DependencyKind::Build => {
                        entry.2.insert(dep.name.clone(), dep_value);
                    }
                }
            }
        }
    }

    if !deps.is_empty() {
        doc.insert("dependencies".into(), Value::Table(deps));
    }
    if !dev_deps.is_empty() {
        doc.insert("dev-dependencies".into(), Value::Table(dev_deps));
    }
    if !build_deps.is_empty() {
        doc.insert("build-dependencies".into(), Value::Table(build_deps));
    }

    // [target.'cfg(...)'.dependencies]
    if !target_deps.is_empty() {
        let mut target_table = toml::map::Map::new();
        for (target_str, (normal, dev, build)) in target_deps {
            let mut t = toml::map::Map::new();
            if !normal.is_empty() {
                t.insert("dependencies".into(), Value::Table(normal));
            }
            if !dev.is_empty() {
                t.insert("dev-dependencies".into(), Value::Table(dev));
            }
            if !build.is_empty() {
                t.insert("build-dependencies".into(), Value::Table(build));
            }
            target_table.insert(target_str, Value::Table(t));
        }
        doc.insert("target".into(), Value::Table(target_table));
    }

    // [features]
    if !package.features.is_empty() {
        let mut features = toml::map::Map::new();
        for (name, feature_list) in &package.features {
            features.insert(
                name.clone(),
                Value::Array(
                    feature_list
                        .iter()
                        .map(|f| Value::String(f.clone()))
                        .collect(),
                ),
            );
        }
        doc.insert("features".into(), Value::Table(features));
    }

    toml::to_string(&Value::Table(doc)).expect("failed to serialize reconstructed Cargo.toml")
}

/// Compute a relative path from the package manifest directory to a target's source path.
fn relative_target_path(package: &Package, target: &Target) -> Option<String> {
    let manifest_dir = package.manifest_path.parent()?;
    let src_path = target.src_path.as_std_path();
    let manifest_dir_path = manifest_dir.as_std_path();
    pathdiff::diff_paths(src_path, manifest_dir_path).map(|p| p.to_string_lossy().into_owned())
}

/// Convert a `Dependency` to a TOML value for Cargo.toml serialization.
fn dependency_to_toml(dep: &Dependency) -> toml::Value {
    use toml::Value;

    let version_str = dep.req.to_string();

    // If the dependency is simple (just a version, no extras), emit as a string
    let has_extras = dep.optional
        || !dep.uses_default_features
        || !dep.features.is_empty()
        || dep.rename.is_some()
        || dep.registry.is_some();

    if !has_extras {
        return Value::String(version_str);
    }

    let mut table = toml::map::Map::new();
    table.insert("version".into(), Value::String(version_str));
    if dep.optional {
        table.insert("optional".into(), Value::Boolean(true));
    }
    if !dep.uses_default_features {
        table.insert("default-features".into(), Value::Boolean(false));
    }
    if !dep.features.is_empty() {
        table.insert(
            "features".into(),
            Value::Array(
                dep.features
                    .iter()
                    .map(|f| Value::String(f.clone()))
                    .collect(),
            ),
        );
    }
    if let Some(ref rename) = dep.rename {
        table.insert("package".into(), Value::String(rename.clone()));
    }
    if let Some(ref registry) = dep.registry {
        table.insert("registry".into(), Value::String(registry.clone()));
    }

    Value::Table(table)
}

/// Normalize a package name such as cargo does.
fn normalize_package_name(package_name: &str) -> String {
    package_name.replace('-', "_")
}

#[derive(Debug)]
/// Helper to retrieve the `ResolvedDependency` structs for a package/crate.
///
/// For this, we need to join the information from `Dependency`, which contains
/// the dependency requirements as specified in `Cargo.toml`, and `NodeDep` which
/// contains the resolved package to use. Unfortunately, there is no simply key
/// on which to perform the join in the general case.
struct ResolvedDependencies<'a> {
    package: &'a Package,
    /// Packages references in the NodeDeps of this package.
    resolved_packages_by_crate_name: HashMap<String, Vec<&'a Package>>,
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

        let mut resolved_packages_by_crate_name: HashMap<String, Vec<&'a Package>> = HashMap::new();
        for node_dep in &node.deps {
            let package = metadata.pkgs_by_id.get(&node_dep.pkg).ok_or_else(|| {
                format_err!(
                    "No matching package for dependency with package id {} in {}.\n-- Package\n{}\n-- Node\n{}",
                    node_dep.pkg,
                    package.id,
                    to_string_pretty(&package).unwrap_or_else(|_| "ERROR".to_string()),
                    to_string_pretty(&node).unwrap_or_else(|_| "ERROR".to_string()),
                )
            })?;
            let packages = resolved_packages_by_crate_name
                .entry(normalize_package_name(&package.name))
                .or_default();
            packages.push(package);
        }
        Ok(ResolvedDependencies {
            package,
            resolved_packages_by_crate_name,
        })
    }

    fn filtered_dependencies(
        &self,
        filter: impl Fn(&Dependency) -> bool,
    ) -> Vec<ResolvedDependency> {
        let ResolvedDependencies {
            package,
            resolved_packages_by_crate_name,
        } = self;

        let mut resolved = package
            .dependencies
            .iter()
            .filter(|package_dep| filter(package_dep))
            .flat_map(|package_dep| {
                let name: String = normalize_package_name(&package_dep.name);
                let resolved = resolved_packages_by_crate_name
                    .get(&name)
                    .and_then(|packages| {
                        let matches = packages
                            .iter()
                            .filter(|p| package_dep.req.matches(&p.version))
                            .collect::<Vec<_>>();

                        // Strip prerelease/build info from versions if we
                        // did not find an exact match.
                        //
                        // E.g. "*" does not match a prerelease version in this
                        // library but cargo thinks differently.
                        let matches = if matches.is_empty() {
                            packages
                                .iter()
                                .filter(|p| {
                                    let without_metadata = {
                                        let mut version = p.version.clone();
                                        version.pre = semver::Prerelease::EMPTY;
                                        version.build = semver::BuildMetadata::EMPTY;
                                        version
                                    };
                                    package_dep.req.matches(&without_metadata)
                                })
                                .collect()
                        } else {
                            matches
                        };

                        // It is possible to have multiple packages that match the name and version
                        // requirement of the dependency. In particular if there are multiple
                        // dependencies on the same package via git at different revisions - in
                        // that case `package_dep.req` is set to `*` so we can't use the version
                        // requirement to match the appropriate locked package with the dependency.
                        // Instead it's necessary to compare by source instead.
                        let matches = if matches.len() > 1 {
                            matches
                                .into_iter()
                                .filter(|p| {
                                    sources_match(package_dep.source.as_deref(), p.source.as_ref())
                                        .unwrap_or(false)
                                })
                                .collect()
                        } else {
                            matches
                        };

                        if matches.len() == 1 {
                            Some(matches[0])
                        } else if matches.is_empty() {
                            None
                        } else {
                            panic!("Could not find an unambiguous package match for dependency, {}. Candidates are: {}", &package_dep.name, matches.iter().map(|p| &p.id).join(", "));
                        }
                    });

                let dep_package = resolved?;

                Some(ResolvedDependency {
                    name: package_dep.name.clone(),
                    rename: package_dep.rename.clone(),
                    package_id: dep_package.id.clone(),
                    target: package_dep.target.clone(),
                    optional: package_dep.optional,
                    uses_default_features: package_dep.uses_default_features,
                    features: package_dep.features.clone(),
                })
            })
            .collect::<Vec<ResolvedDependency>>();

        resolved.sort_by(|d1, d2| d1.package_id.cmp(&d2.package_id));
        resolved
    }
}

fn sources_match(
    dependency_source: Option<&str>,
    package_source: Option<&Source>,
) -> Result<bool, anyhow::Error> {
    let Some(dependency_source) = dependency_source else {
        return Ok(package_source.is_none());
    };
    let Some(package_source) = package_source else {
        return Ok(false); // fail if dependency has a source, but package does not
    };

    let dependency = Url::parse(dependency_source)?;
    let package = Url::parse(&package_source.repr)?;

    let scheme_matches = dependency.scheme() == package.scheme();
    let domain_matches = dependency.domain() == package.domain();
    let path_matches = dependency.path() == package.path();
    let query_matches = {
        let package_query = package.query_pairs().collect::<HashMap<_, _>>();
        dependency.query_pairs().all(|(key, dep_value)| {
            package_query
                .get(&key)
                .is_some_and(|pkg_value| &dep_value == pkg_value)
        })
    };

    Ok(scheme_matches && domain_matches && path_matches && query_matches)
}

/// Converts one type into another by serializing/deserializing it.
///
/// Therefore, the output json of `I` must be deserializable to `O`.
#[allow(unused)]
#[cfg(test)]
fn serialize_deserialize<I: Serialize, O>(input: &I) -> O
where
    for<'d> O: Deserialize<'d>,
{
    let json_string = serde_json::to_string(input).expect("serialize");
    let deserialized: Result<O, _> = serde_json::from_str(&json_string);
    deserialized.expect("deserialize")
}

#[test]
pub fn resolved_dependencies_new_with_double_crate() {
    let mut env = test::MetadataEnv::default();

    let mut main = env.add_package_and_node("main");
    main.make_root();
    main.add_dependency("futures")
        .version_and_package_id("0.1.0")
        .update_package_dep(|d| d.rename = Some("futures01".to_string()))
        .update_node_dep(|n| n.name = "futures01".to_string());
    main.add_dependency("futures")
        .version_and_package_id("0.3.0")
        .update_package_dep(|d| {
            d.uses_default_features = false;
            d.features = vec!["compat".to_string()];
        });

    let indexed = env.indexed_metadata();

    let root_package = &indexed.root_package().expect("root package");
    let resolved_deps = ResolvedDependencies::new(&indexed, root_package).unwrap();

    assert_eq!(
        resolved_deps.resolved_packages_by_crate_name.len(),
        1,
        "unexpected packages_by_crate_name: {}",
        serde_json::to_string_pretty(&resolved_deps.resolved_packages_by_crate_name).unwrap()
    );
    assert!(
        resolved_deps
            .resolved_packages_by_crate_name
            .contains_key("futures"),
        "unexpected packages_by_crate_name: {}",
        serde_json::to_string_pretty(&resolved_deps.resolved_packages_by_crate_name).unwrap()
    );
    assert_eq!(
        resolved_deps
            .resolved_packages_by_crate_name
            .get("futures")
            .unwrap()
            .len(),
        2,
        "unexpected packages_by_crate_name: {}",
        serde_json::to_string_pretty(&resolved_deps.resolved_packages_by_crate_name).unwrap()
    );

    env.close();
}

#[test]
pub fn resolved_dependencies_filtered_dependencies_with_double_crate() {
    let mut env = test::MetadataEnv::default();

    let mut main = env.add_package_and_node("main");
    main.make_root();
    main.add_dependency("futures")
        .version_and_package_id("0.1.0")
        .update_package_dep(|d| d.rename = Some("futures01".to_string()))
        .update_node_dep(|n| n.name = "futures01".to_string());
    main.add_dependency("futures")
        .version_and_package_id("0.3.0")
        .update_package_dep(|d| {
            d.uses_default_features = false;
            d.features = vec!["compat".to_string()];
        });

    let indexed = env.indexed_metadata();

    let root_package = &indexed.root_package().expect("root package");
    let resolved_deps = ResolvedDependencies::new(&indexed, root_package).unwrap();

    let filtered_deps = resolved_deps.filtered_dependencies(|d| {
        d.kind == DependencyKind::Normal || d.kind == DependencyKind::Unknown
    });

    assert_eq!(
        filtered_deps.len(),
        2,
        "unexpected resolved dependencies: {}",
        serde_json::to_string_pretty(&filtered_deps).unwrap()
    );

    env.close();
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResolvedDependency {
    pub name: String,
    /// New name for the dependency if it is renamed.
    pub rename: Option<String>,
    pub package_id: PackageId,
    /// The cfg expression for conditionally enabling the dependency (if any).
    /// Can also be a target "triplet".
    pub target: Option<Platform>,
    /// Whether this dependency is optional and thus needs to be enabled via a feature.
    pub optional: bool,
    /// Whether the crate uses this dependency with default features enabled.
    pub uses_default_features: bool,
    /// Extra-enabled features.
    pub features: Vec<String>,
}

#[cfg(test)]
mod reconstruct_cargo_toml_tests {
    use super::*;

    fn make_package(json: serde_json::Value) -> Package {
        serde_json::from_value(json).expect("invalid test Package JSON")
    }

    #[test]
    fn basic_package_fields() {
        let pkg = make_package(serde_json::json!({
            "name": "my-crate",
            "version": "1.2.3",
            "id": "my-crate 1.2.3",
            "edition": "2021",
            "authors": ["Alice <alice@example.com>"],
            "description": "A test crate",
            "license": "MIT",
            "manifest_path": "/tmp/Cargo.toml",
            "dependencies": [],
            "targets": [],
            "features": {},
        }));

        let toml_str = reconstruct_cargo_toml(&pkg);
        let parsed: toml::Value = toml::from_str(&toml_str).expect("invalid TOML output");
        let table = parsed.as_table().unwrap();
        let package = table["package"].as_table().unwrap();

        assert_eq!(package["name"].as_str().unwrap(), "my-crate");
        assert_eq!(package["version"].as_str().unwrap(), "1.2.3");
        assert_eq!(package["edition"].as_str().unwrap(), "2021");
        assert_eq!(package["description"].as_str().unwrap(), "A test crate");
        assert_eq!(package["license"].as_str().unwrap(), "MIT");
        let authors = package["authors"].as_array().unwrap();
        assert_eq!(authors[0].as_str().unwrap(), "Alice <alice@example.com>");
    }

    #[test]
    fn simple_dependencies() {
        let pkg = make_package(serde_json::json!({
            "name": "my-crate",
            "version": "0.1.0",
            "id": "my-crate 0.1.0",
            "manifest_path": "/tmp/Cargo.toml",
            "dependencies": [
                {
                    "name": "serde",
                    "req": "^1.0",
                    "kind": null,
                    "optional": false,
                    "uses_default_features": true,
                    "features": [],
                    "target": null,
                    "rename": null,
                    "registry": null,
                    "source": null,
                }
            ],
            "targets": [],
            "features": {},
        }));

        let toml_str = reconstruct_cargo_toml(&pkg);
        let parsed: toml::Value = toml::from_str(&toml_str).expect("invalid TOML output");
        let deps = parsed["dependencies"].as_table().unwrap();
        // Simple dep with no extras should be a string
        assert_eq!(deps["serde"].as_str().unwrap(), "^1.0");
    }

    #[test]
    fn dependency_with_features_and_optional() {
        let pkg = make_package(serde_json::json!({
            "name": "my-crate",
            "version": "0.1.0",
            "id": "my-crate 0.1.0",
            "manifest_path": "/tmp/Cargo.toml",
            "dependencies": [
                {
                    "name": "serde",
                    "req": "^1.0",
                    "kind": null,
                    "optional": true,
                    "uses_default_features": false,
                    "features": ["derive", "alloc"],
                    "target": null,
                    "rename": null,
                    "registry": null,
                    "source": null,
                }
            ],
            "targets": [],
            "features": {},
        }));

        let toml_str = reconstruct_cargo_toml(&pkg);
        let parsed: toml::Value = toml::from_str(&toml_str).expect("invalid TOML output");
        let dep = parsed["dependencies"]["serde"].as_table().unwrap();
        assert_eq!(dep["version"].as_str().unwrap(), "^1.0");
        assert_eq!(dep["optional"].as_bool().unwrap(), true);
        assert_eq!(dep["default-features"].as_bool().unwrap(), false);
        let features: Vec<&str> = dep["features"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(features, vec!["derive", "alloc"]);
    }

    #[test]
    fn dependency_with_rename() {
        let pkg = make_package(serde_json::json!({
            "name": "my-crate",
            "version": "0.1.0",
            "id": "my-crate 0.1.0",
            "manifest_path": "/tmp/Cargo.toml",
            "dependencies": [
                {
                    "name": "futures",
                    "req": "^0.1",
                    "kind": null,
                    "optional": false,
                    "uses_default_features": true,
                    "features": [],
                    "target": null,
                    "rename": "futures01",
                    "registry": null,
                    "source": null,
                }
            ],
            "targets": [],
            "features": {},
        }));

        let toml_str = reconstruct_cargo_toml(&pkg);
        let parsed: toml::Value = toml::from_str(&toml_str).expect("invalid TOML output");
        let dep = parsed["dependencies"]["futures"].as_table().unwrap();
        assert_eq!(dep["version"].as_str().unwrap(), "^0.1");
        assert_eq!(dep["package"].as_str().unwrap(), "futures01");
    }

    #[test]
    fn build_and_dev_dependencies() {
        let pkg = make_package(serde_json::json!({
            "name": "my-crate",
            "version": "0.1.0",
            "id": "my-crate 0.1.0",
            "manifest_path": "/tmp/Cargo.toml",
            "dependencies": [
                {
                    "name": "cc",
                    "req": "^1.0",
                    "kind": "build",
                    "optional": false,
                    "uses_default_features": true,
                    "features": [],
                    "target": null,
                    "rename": null,
                    "registry": null,
                    "source": null,
                },
                {
                    "name": "tempdir",
                    "req": "^0.3",
                    "kind": "dev",
                    "optional": false,
                    "uses_default_features": true,
                    "features": [],
                    "target": null,
                    "rename": null,
                    "registry": null,
                    "source": null,
                }
            ],
            "targets": [],
            "features": {},
        }));

        let toml_str = reconstruct_cargo_toml(&pkg);
        let parsed: toml::Value = toml::from_str(&toml_str).expect("invalid TOML output");
        assert_eq!(parsed["build-dependencies"]["cc"].as_str().unwrap(), "^1.0");
        assert_eq!(
            parsed["dev-dependencies"]["tempdir"].as_str().unwrap(),
            "^0.3"
        );
        // Should not have regular dependencies section
        assert!(parsed.get("dependencies").is_none());
    }

    #[test]
    fn target_specific_dependencies() {
        let pkg = make_package(serde_json::json!({
            "name": "my-crate",
            "version": "0.1.0",
            "id": "my-crate 0.1.0",
            "manifest_path": "/tmp/Cargo.toml",
            "dependencies": [
                {
                    "name": "libc",
                    "req": "^0.2",
                    "kind": null,
                    "optional": false,
                    "uses_default_features": true,
                    "features": [],
                    "target": "cfg(unix)",
                    "rename": null,
                    "registry": null,
                    "source": null,
                }
            ],
            "targets": [],
            "features": {},
        }));

        let toml_str = reconstruct_cargo_toml(&pkg);
        let parsed: toml::Value = toml::from_str(&toml_str).expect("invalid TOML output");
        let target_deps = parsed["target"]["cfg(unix)"]["dependencies"]
            .as_table()
            .unwrap();
        assert_eq!(target_deps["libc"].as_str().unwrap(), "^0.2");
    }

    #[test]
    fn features_section() {
        let pkg = make_package(serde_json::json!({
            "name": "my-crate",
            "version": "0.1.0",
            "id": "my-crate 0.1.0",
            "manifest_path": "/tmp/Cargo.toml",
            "dependencies": [],
            "targets": [],
            "features": {
                "default": ["std"],
                "std": [],
                "alloc": ["dep:serde"],
            },
        }));

        let toml_str = reconstruct_cargo_toml(&pkg);
        let parsed: toml::Value = toml::from_str(&toml_str).expect("invalid TOML output");
        let features = parsed["features"].as_table().unwrap();
        let default_features: Vec<&str> = features["default"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        assert_eq!(default_features, vec!["std"]);
        assert!(features["std"].as_array().unwrap().is_empty());
    }

    #[test]
    fn output_is_valid_toml() {
        let pkg = make_package(serde_json::json!({
            "name": "complex-crate",
            "version": "2.0.0-alpha.1",
            "id": "complex-crate 2.0.0-alpha.1",
            "edition": "2021",
            "authors": ["Bob <bob@test.com>"],
            "manifest_path": "/tmp/Cargo.toml",
            "dependencies": [
                {
                    "name": "serde",
                    "req": "^1.0",
                    "kind": null,
                    "optional": true,
                    "uses_default_features": false,
                    "features": ["derive"],
                    "target": null,
                    "rename": null,
                    "registry": null,
                    "source": null,
                },
                {
                    "name": "libc",
                    "req": "^0.2",
                    "kind": null,
                    "optional": false,
                    "uses_default_features": true,
                    "features": [],
                    "target": "cfg(unix)",
                    "rename": null,
                    "registry": null,
                    "source": null,
                },
                {
                    "name": "cc",
                    "req": "^1",
                    "kind": "build",
                    "optional": false,
                    "uses_default_features": true,
                    "features": [],
                    "target": null,
                    "rename": null,
                    "registry": null,
                    "source": null,
                },
            ],
            "targets": [],
            "features": {
                "default": ["serde"],
                "full": ["serde", "std"],
            },
        }));

        let toml_str = reconstruct_cargo_toml(&pkg);
        // Should parse without error
        let _: toml::Value = toml::from_str(&toml_str).expect("invalid TOML output");
    }
}
