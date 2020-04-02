///! Constructor functions for test data.
use cargo_metadata::{Dependency, Metadata, Node, NodeDep, Package, PackageId, Resolve};
use tempdir::TempDir;

/// Returns bogus crate::GenerateConfig.
pub fn generate_config() -> crate::GenerateConfig {
    crate::GenerateConfig {
        cargo_toml: "Cargo.toml".into(),
        crate_hashes_json: "crate-hashes.json".into(),
        nixpkgs_path: "bogus-nixpkgs-path".into(),
        other_metadata_options: vec![],
        output: "Cargo.nix".into(),
        use_cargo_lock_checksums: true,
        read_crate_hashes: true,
    }
}

#[derive(Debug)]
pub struct MetadataEnv {
    /// Keep track of temporary directories.
    temp_dirs: Vec<TempDir>,
    metadata: Metadata,
}

impl Drop for MetadataEnv {
    fn drop(&mut self) {
        if !self.temp_dirs.is_empty() {
            eprintln!(
                "Consider explicitly closing MetadataEnv: {:?}",
                self.temp_dirs
            );
        }
    }
}

impl Default for MetadataEnv {
    fn default() -> Self {
        MetadataEnv {
            temp_dirs: Vec::new(),
            metadata: empty_metadata(),
        }
    }
}

impl MetadataEnv {
    fn resolve(&mut self) -> &mut Resolve {
        self.metadata.resolve.get_or_insert_with(|| empty_resolve())
    }

    pub fn add_package_and_node(&mut self, name: &str) -> PackageAndNode {
        let package = package(name, "0.1.0");
        let package_idx = self.metadata.packages.len();
        self.metadata.packages.push(package.clone());
        let nodes = &mut self.resolve().nodes;
        let node_idx = nodes.len();
        nodes.push(node(&package.id.repr));
        self.temp_dirs.push(package.crate_dir);
        PackageAndNode {
            env: self,
            package_idx,
            node_idx,
        }
    }

    pub fn metadata(&self) -> Metadata {
        self.metadata.clone()
    }

    pub fn indexed_metadata(&self) -> crate::IndexedMetadata {
        crate::IndexedMetadata::new_from(self.metadata()).unwrap()
    }

    pub fn close(&mut self) {
        for temp_dir in self.temp_dirs.drain(..) {
            temp_dir.close().expect("while closing temp");
        }
    }
}

#[derive(Debug)]
pub struct PackageAndNode<'a> {
    env: &'a mut MetadataEnv,
    package_idx: usize,
    node_idx: usize,
}

impl<'a> PackageAndNode<'a> {
    pub fn make_root(&mut self) -> &mut Self {
        self.env.resolve().root = Some(self.get_mut_package().id.clone());
        self
    }

    pub fn get_mut_package(&mut self) -> &mut Package {
        &mut self.env.metadata.packages[self.package_idx]
    }
    pub fn get_package(&self) -> &Package {
        &self.env.metadata.packages[self.package_idx]
    }
    pub fn update_package(&mut self, f: impl FnOnce(&mut Package)) -> &mut Self {
        f(self.get_mut_package());
        self
    }
    pub fn get_mut_node(&mut self) -> &mut Node {
        &mut self.env.resolve().nodes[self.node_idx]
    }
    pub fn get_node(&mut self) -> &Node {
        &self.env.resolve().nodes[self.node_idx]
    }
    pub fn update_node(&mut self, f: impl FnOnce(&mut Node)) -> &mut Self {
        f(self.get_mut_node());
        self
    }
    pub fn version_and_package_id(&mut self, version: &str) -> &mut Self {
        let p = self.get_mut_package();
        p.version = semver::Version::parse(version).expect("version incorrect");
        p.id = crates_io_package_id(&p.name, version);
        self
    }
    pub fn add_dependency<'b>(&'b mut self, name: &str) -> PackageAndNodeDep<'b> {
        let package_dep_idx = self.get_mut_package().dependencies.len();
        let node_dep_idx = self.get_mut_node().dependencies.len();

        let mut new_package = self.env.add_package_and_node(name);
        let pkg_dep = dependency_from_package(new_package.get_mut_package());
        let node_dep = node_dep(name, &new_package.get_mut_package().id);
        let PackageAndNode {
            package_idx,
            node_idx,
            ..
        } = new_package;

        self.get_mut_package().dependencies.push(pkg_dep);
        let node = self.get_mut_node();
        node.dependencies.push(node_dep.pkg.clone());
        node.deps.push(node_dep);

        PackageAndNodeDep {
            package_and_node: PackageAndNode {
                env: self.env,
                package_idx,
                node_idx,
            },
            parent_package_idx: self.package_idx,
            parent_node_idx: self.node_idx,
            package_dep_idx,
            node_dep_idx,
        }
    }
}

#[derive(Debug)]
pub struct PackageAndNodeDep<'a> {
    package_and_node: PackageAndNode<'a>,
    package_dep_idx: usize,
    node_dep_idx: usize,
    parent_package_idx: usize,
    parent_node_idx: usize,
}

impl<'a> PackageAndNodeDep<'a> {
    pub fn update_package_and_node(&mut self, f: impl FnOnce(&mut PackageAndNode)) -> &mut Self {
        f(&mut self.package_and_node);
        self
    }
    pub fn get_package(&self) -> &Package {
        self.package_and_node.get_package()
    }
    pub fn update_package(&mut self, f: impl FnOnce(&mut Package)) -> &mut Self {
        self.package_and_node.update_package(f);
        self
    }
    pub fn update_node(&mut self, f: impl FnOnce(&mut Node)) -> &mut Self {
        self.package_and_node.update_node(f);
        self
    }
    fn get_mut_parent_package(&mut self) -> &mut Package {
        let parent_package_idx = self.parent_package_idx;
        &mut self.package_and_node.env.metadata.packages[parent_package_idx]
    }
    pub fn get_mut_package_dep(&mut self) -> &mut Dependency {
        let package_dep_idx = self.package_dep_idx;
        &mut self.get_mut_parent_package().dependencies[package_dep_idx]
    }
    pub fn update_package_dep(&mut self, f: impl FnOnce(&mut Dependency)) -> &mut Self {
        f(self.get_mut_package_dep());
        self
    }
    fn get_mut_parent_node(&mut self) -> &mut Node {
        let parent_node_idx = self.parent_node_idx;
        &mut self.package_and_node.env.resolve().nodes[parent_node_idx]
    }
    pub fn get_mut_node_dep(&mut self) -> &mut NodeDep {
        let node_dep_idx = self.node_dep_idx;
        &mut self.get_mut_parent_node().deps[node_dep_idx]
    }
    pub fn update_node_dep(&mut self, f: impl FnOnce(&mut NodeDep)) -> &mut Self {
        f(self.get_mut_node_dep());
        self
    }
    pub fn version_and_package_id(&mut self, version: &str) -> &mut Self {
        self.update_package_and_node(|pn| {
            pn.version_and_package_id(version);
        });
        let pkg_id = self.package_and_node.get_mut_package().id.clone();
        self.update_node(|n| n.id = pkg_id.clone());
        self.update_package_dep(|d| {
            d.req = semver::VersionReq::parse(&format!("={}", version)).unwrap();
        });
        let node_dep_idx = self.node_dep_idx;
        self.get_mut_parent_node().dependencies[node_dep_idx] = pkg_id.clone();
        self.update_node_dep(|n| {
            n.pkg = pkg_id.clone();
        });
        self
    }
}

#[derive(Debug)]
#[must_use = "Please close."]
pub struct TestPackage {
    inner: Package,
    crate_dir: tempdir::TempDir,
}

impl TestPackage {
    pub fn close(self) -> Result<(), std::io::Error> {
        self.crate_dir.close()
    }
}

impl std::ops::Deref for TestPackage {
    type Target = Package;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for TestPackage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

fn from_value_unwrap<T>(json: impl Fn() -> serde_json::Value) -> T
where
    T: serde::de::DeserializeOwned,
{
    use serde_json::{from_value, to_string_pretty};
    from_value(json()).expect(&format!(
        "invalid {}: {}",
        std::any::type_name::<T>(),
        to_string_pretty(&json()).unwrap()
    ))
}

/// Return value from given JSON.
///
/// If deserialization fails, the error message includes the JSON itself.
#[macro_export]
macro_rules! from_json {
    // Hide distracting implementation details from the generated rustdoc.
    ($($json:tt)+) => {
        from_value_unwrap(|| serde_json::json!($($json)+))
    };
}

pub fn crates_io_package_id(name: &str, version: &str) -> PackageId {
    PackageId {
        repr: format!(
            "{} {} (registry+https://github.com/rust-lang/crates.io-index)",
            name, version
        ),
    }
}

/// Returns a package with minimal bogus data necessary so that
/// this is a valid package.
pub fn package(name: &str, version: &str) -> TestPackage {
    semver::Version::parse(version).expect("invalid version");
    let crate_dir: TempDir =
        TempDir::new(&format!("crate2nix_crate_{}_{}", name, version)).expect("test dir creation");

    TestPackage {
        inner: from_json!(
        {
            "name": name,
            "version": version,
            "id": crates_io_package_id(name, version),
            "manifest_path":
                format!("{}/Cargo.toml", crate_dir.path().to_str().unwrap()),
            "dependencies": [],
            "targets": [],
            "features": {},
        }),
        crate_dir,
    }
}

pub fn bin_target(package: &TestPackage, file: &str) -> cargo_metadata::Target {
    from_json!(
        {
            "kind": [
              "bin"
            ],
            "crate_types": [
              "bin"
            ],
            "name": if file == "main" { &package.name } else { file },
            "src_path": format!("{}/src/{}.rs", package.crate_dir.path().to_str().unwrap(), file),
            "edition": "2018",
            "doctest": false
          }

    )
}

pub fn lib_target(package: &TestPackage, file: &str) -> cargo_metadata::Target {
    from_json!(
        {
            "kind": [
                "lib"
            ],
            "crate_types": [
                "lib"
            ],
            "name": if file == "lib" { &package.name } else { file },
            "src_path": format!("{}/src/{}.rs", package.crate_dir.path().to_str().unwrap(), file),
            "edition": "2018",
            "doctest": true
        }

    )
}

/// Returns a bin package with minimal bogus data necessary so that
/// this is a valid package.
pub fn bin_package(name: &str, version: &str) -> TestPackage {
    let mut package = package(name, version);
    package.targets = vec![bin_target(&package, "main")];
    package
}

/// Returns a lib package with minimal bogus data necessary so that
/// this is a valid package.
pub fn lib_package(name: &str, version: &str) -> TestPackage {
    let mut package = package(name, version);
    package.targets = vec![lib_target(&package, "lib")];
    package
}

pub fn dependency_from_package(package: &Package) -> Dependency {
    from_json!(
        {
            "name": &package.name,
            "source": &package.source,
            "req": format!("={}", package.version),
            "kind": null,
            "rename": null,
            "optional": false,
            "uses_default_features": true,
            "features": [],
            "target": null,
            "registry": null
          }
    )
}

#[test]
pub fn test_package() {
    println!("{:#?}", package("test", "0.1.0"));
}

pub fn node(package_id: &str) -> Node {
    from_json!({
          "id": package_id,
          "dependencies": [],
          "deps": [],
          "features": []
        }
    )
}

pub fn node_dep(name: &str, package_id: &PackageId) -> NodeDep {
    from_json!({
        "name": name,
        "pkg": package_id,
        "dep_kinds": [
          {
            "kind": null,
            "target": null
          }
        ]
      }
    )
}

pub fn node_with_deps(package_id: &str, deps: Vec<NodeDep>) -> Node {
    let mut node = node(package_id);
    node.dependencies = deps.iter().map(|d| &d.pkg).cloned().collect();
    node.deps = deps;
    node
}

#[test]
pub fn test_valid_node() {
    println!("{:#?}", node("bogus package id"));
}

pub fn empty_resolve() -> Resolve {
    from_json!({
            "nodes": [],
    })
}

#[test]
pub fn test_valid_empty_resolve() {
    println!("{:#?}", empty_resolve());
}

pub fn empty_metadata() -> Metadata {
    from_json!({
        "version": 1,
        "packages": [],
        "workspace_members": [],
        "workspace_root": "",
        "target_directory": "",
    })
}

#[test]
pub fn test_valid_empty_metadata() {
    println!("{:#?}", empty_metadata());
}
