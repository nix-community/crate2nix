use std::path::Path;

use tempdir::TempDir;

use cargo2nix::default_nix;
use cargo2nix::nix_build::nix_build;
use cargo2nix::nix_build::run_cmd;
use cargo2nix::render;
use cargo2nix::GenerateConfig;
use fs_extra::dir::CopyOptions;

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

#[test]
fn build_and_run_with_tera() {
    let output = build_and_run("sample_projects/with_tera/Cargo.toml");

    assert_eq!("Hello, with_tera!\n", &output);
}

fn build_and_run(cargo_toml: impl AsRef<Path>) -> String {
    let orig_project_dir = cargo_toml
        .as_ref()
        .parent()
        .expect("Cargo.toml needs a parent")
        .to_path_buf();

    let temp_dir = TempDir::new("sample_projects").expect("couldn't create temp dir");

    fs_extra::dir::copy("sample_projects", &temp_dir, &CopyOptions::new())
        .expect("while copying files");
    let cargo_toml = temp_dir.path().join(cargo_toml);
    let project_dir = cargo_toml
        .parent()
        .expect("Cargo.toml needs a parent")
        .to_path_buf();

    // Get metadata
    let metadata = default_nix(&GenerateConfig {
        cargo_toml: cargo_toml.clone(),
        nixpkgs_path: "<nixos-unstable>".to_string(),
        crate_hashes_json: project_dir.join("crate-hashes.json").to_path_buf(),
    })
    .unwrap();
    let default_nix_content = render::default_nix(&metadata).unwrap();

    // Generate nix file
    let default_nix_path = cargo_toml.parent().unwrap().join("default.nix");
    render::write_to_file(default_nix_path, &default_nix_content).unwrap();

    // Copy lock files back to source to avoid expensive, repetitive work
    fs_extra::copy_items(
        &vec![
            project_dir.join("Cargo.lock"),
            project_dir.join("crate-hashes.json"),
        ],
        orig_project_dir,
        &CopyOptions {
            overwrite: true,
            ..CopyOptions::new()
        },
    )
    .unwrap();

    // Build
    nix_build(&project_dir).unwrap();

    // Run resulting binary
    let binary_name = metadata
        .indexed_metadata
        .pkgs_by_id
        .get(&metadata.root_derivation_name.unwrap().clone())
        .unwrap()
        .name
        .clone();
    let bin_path = project_dir.join("result").join("bin").join(&binary_name);
    let output = run_cmd(bin_path).unwrap();
    temp_dir.close().expect("couldn't remove temp dir");
    output
}
