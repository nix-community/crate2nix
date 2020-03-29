///! Constructor functions for test data.
use cargo_metadata::Node;
use cargo_metadata::Package;
use cargo_metadata::{Metadata, Resolve};
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

/// Returns a package with minimal bogus data necessary so that
/// this is a valid package.
pub fn package(name: &str, version: &str) -> TestPackage {
    use serde_json::{from_value, json, to_string_pretty};

    semver::Version::parse(version).expect("invalid version");
    let crate_dir: TempDir =
        TempDir::new(&format!("crate2nix_crate_{}_{}", name, version)).expect("test dir creation");
    let package_json = || {
        json!({
            "name": name,
            "version": version,
            "id": &format!("{} {} (registry+https://github.com/rust-lang/crates.io-index)", name, version),
            "manifest_path":
                &format!("{}/Cargo.toml", crate_dir.path().to_str().unwrap()),
            "dependencies": [],
            "targets": [],
            "features": {},
        })
    };
    let package = from_value(package_json()).expect(&format!(
        "package_json invalid: {}",
        to_string_pretty(&package_json()).unwrap()
    ));

    TestPackage {
        inner: package,
        crate_dir,
    }
}

#[test]
pub fn test_package() {
    println!("{:#?}", package("test", "0.1.0"));
}

pub fn node(package_id: &str) -> Node {
    use serde_json::{from_value, json, to_string_pretty};

    let node_json = || {
        json!({
          "id": package_id,
          "dependencies": [],
          "deps": [],
          "features": []
        })
    };
    from_value(node_json()).expect(&format!(
        "node_json invalid: {}",
        to_string_pretty(&node_json()).unwrap()
    ))
}

#[test]
pub fn test_valid_node() {
    println!("{:#?}", node("bogus package id"));
}

pub fn empty_resolve() -> Resolve {
    use serde_json::{from_value, json, to_string_pretty};

    let empty_resolve_json = || {
        json!({
            "nodes": [],
        })
    };
    from_value(empty_resolve_json()).expect(&format!(
        "empty_resolve invalid: {}",
        to_string_pretty(&empty_resolve_json()).unwrap()
    ))
}

#[test]
pub fn test_valid_empty_resolve() {
    println!("{:#?}", empty_resolve());
}

pub fn empty_metadata() -> Metadata {
    use serde_json::{from_value, json, to_string_pretty};

    let node_json = || {
        json!({
            "version": 1,
            "packages": [],
            "workspace_members": [],
            "workspace_root": "",
            "target_directory": "",
        })
    };
    from_value(node_json()).expect(&format!(
        "node_json invalid: {}",
        to_string_pretty(&node_json()).unwrap()
    ))
}

#[test]
pub fn test_valid_empty_metadata() {
    println!("{:#?}", empty_metadata());
}
