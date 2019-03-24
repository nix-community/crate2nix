use std::path::Path;

use tempdir::TempDir;

use cargo2nix::default_nix;
use cargo2nix::nix_build::nix_build;
use cargo2nix::nix_build::run_cmd;
use cargo2nix::render;
use cargo2nix::GenerateConfig;

#[test]
fn build_and_run_bin() {
    let output = build_and_run("sample_projects/bin/Cargo.toml");

    assert_eq!("Hello, world!\n", &output);
}

#[test]
fn build_and_run_lib_and_bin() {
    let output = build_and_run("sample_projects/lib_and_bin/Cargo.toml");

    assert_eq!("Hello, lib_and_bin!\n", &output);
}

#[test]
fn build_and_run_bin_with_lib_dep() {
    let output = build_and_run("sample_projects/bin_with_lib_dep/Cargo.toml");

    assert_eq!("Hello, bin_with_lib_dep!\n", &output);
}

#[test]
fn build_and_run_with_default_features() {
    let output = build_and_run("sample_projects/bin_with_default_features/Cargo.toml");

    assert_eq!("Hello, bin_with_default_features!\n", &output);
}

fn build_and_run(cargo_toml: impl AsRef<Path>) -> String {
    // Get metadata
    let metadata = default_nix(&GenerateConfig {
        cargo_toml: cargo_toml.as_ref().to_path_buf(),
        nixpkgs_path: "<nixos-unstable>".to_string(),
        crate_hashes_json: cargo_toml
            .as_ref()
            .parent()
            .expect("Cargo.toml needs a parent")
            .to_path_buf()
            .join("crate-hashes.json"),
    })
    .unwrap();
    let default_nix_content = render::default_nix(&metadata).unwrap();

    // Generate nix file
    let temp_dir = TempDir::new("hello_world").expect("couldn't create temp dir");
    let file_path = temp_dir.path().join("default.nix");
    render::write_to_file(file_path, &default_nix_content).unwrap();

    // Build
    nix_build(temp_dir.path()).unwrap();

    // Run resulting binary
    let binary_name = metadata
        .indexed_metadata
        .pkgs_by_id
        .get(&metadata.root_derivation_name.unwrap().clone())
        .unwrap()
        .name
        .clone();
    let bin_path = temp_dir
        .path()
        .join("result")
        .join("bin")
        .join(&binary_name);
    let output = run_cmd(bin_path).unwrap();
    temp_dir.close().expect("couldn't remove temp dir");
    output
}
