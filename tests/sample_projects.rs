use std::path::{Path, PathBuf};
use std::time::Instant;

use tempdir::TempDir;

use crate2nix::nix_build::nix_build;
use crate2nix::nix_build::{dump_with_lines, run_cmd};
use crate2nix::{render, BuildInfo};
use crate2nix::{GenerateConfig, GenerateInfo};
use fs_extra::dir::CopyOptions;

#[test]
fn build_and_run_bin() {
    let output = build_and_run(
        "sample_projects/bin/Cargo.toml",
        "sample_projects/bin",
        "rootCrate",
        "hello_world_bin",
        &["default"],
    );

    assert_eq!("Hello, world!\n", &output);
}

#[test]
fn build_and_run_bin_with_deprecated_alias() {
    let output = build_and_run(
        "sample_projects/bin/Cargo.toml",
        "sample_projects/bin",
        "root_crate",
        "hello_world_bin",
        &["default"],
    );

    assert_eq!("Hello, world!\n", &output);
}

#[test]
fn build_and_run_lib_and_bin() {
    let output = build_and_run(
        "sample_projects/lib_and_bin/Cargo.toml",
        "sample_projects/lib_and_bin",
        "rootCrate",
        "hello_world_lib_and_bin",
        &["default"],
    );

    assert_eq!("Hello, lib_and_bin!\n", &output);
}

#[test]
fn build_and_run_bin_with_lib_dep() {
    let output = build_and_run(
        "sample_projects/bin_with_lib_dep/Cargo.toml",
        "sample_projects",
        "rootCrate",
        "hello_world_with_dep",
        &["default"],
    );

    assert_eq!("Hello, bin_with_lib_dep!\n", &output);
}

#[test]
fn build_and_run_with_default_features() {
    let output = build_and_run(
        "sample_projects/bin_with_default_features/Cargo.toml",
        "sample_projects",
        "rootCrate",
        "bin_with_default_features",
        &["default"],
    );

    assert_eq!("Hello, bin_with_default_features!\n", &output);
}

#[test]
fn build_and_run_with_non_default_features() {
    let output = build_and_run(
        "sample_projects/bin_with_default_features/Cargo.toml",
        "sample_projects",
        "rootCrate",
        "bin_with_default_features",
        &["default", "do_not_activate"],
    );

    assert_eq!(
        "Hello, bin_with_default_features!\nCOMPILED with do_not_activate\n",
        &output
    );
}

#[test]
fn build_and_run_with_problematic_crates() {
    let output = build_and_run(
        "sample_projects/with_problematic_crates/Cargo.toml",
        "sample_projects/with_problematic_crates",
        "rootCrate",
        "with_problematic_crates",
        &["default"],
    );

    assert_eq!("Hello, with_problematic_crates!\n", &output);
}

#[test]
fn build_and_run_bin_with_lib_git_dep() {
    let output = build_and_run(
        "sample_projects/bin_with_lib_git_dep/Cargo.toml",
        "sample_projects/bin_with_lib_git_dep",
        "rootCrate",
        "bin_with_lib_git_dep",
        &["default"],
    );

    assert_eq!("Hello world from bin_with_lib_git_dep!\n", &output);
}

#[test]
#[ignore]
fn build_and_run_bin_with_rerenamed_lib_dep() {
    let output = build_and_run(
        "sample_projects/bin_with_rerenamed_lib_dep/Cargo.toml",
        "sample_projects",
        "rootCrate.build",
        "bin_with_rerenamed_lib_dep",
        &["default"],
    );

    assert_eq!("Hello world from bin_with_rerenamed_lib_dep!\n", &output);
}

#[test]
fn build_and_run_workspace() {
    let output = build_and_run(
        "sample_workspace/Cargo.toml",
        "sample_workspace",
        "workspaceMembers.with_tera.build",
        "with_tera",
        &["default"],
    );

    assert_eq!("Hello, with_tera!\n", &output);
}

#[test]
fn build_and_run_workspace_with_deprecated_alias() {
    let output = build_and_run(
        "sample_workspace/Cargo.toml",
        "sample_workspace",
        "workspace_members.with_tera",
        "with_tera",
        &["default"],
    );

    assert_eq!("Hello, with_tera!\n", &output);
}

fn build_and_run(
    cargo_toml: impl AsRef<Path>,
    copy_dir: impl AsRef<Path>,
    nix_attr: &str,
    binary_name: &str,
    features: &[&str],
) -> String {
    let orig_project_dir = cargo_toml
        .as_ref()
        .parent()
        .expect("Cargo.toml needs a parent")
        .to_path_buf();

    let temp_dir = TempDir::new(&copy_dir.as_ref().file_name().unwrap().to_string_lossy())
        .expect("couldn't create temp dir");

    eprintln!(
        "Created temp_dir for test at {}",
        temp_dir.path().to_string_lossy()
    );

    fs_extra::dir::copy(copy_dir.as_ref(), &temp_dir, &CopyOptions::new())
        .expect("while copying files");
    let relative_cargo_toml: &Path = cargo_toml
        .as_ref()
        .strip_prefix(copy_dir.as_ref().parent().unwrap())
        .expect("prefix");
    let cargo_toml = temp_dir.path().join(relative_cargo_toml);
    let project_dir = cargo_toml
        .parent()
        .expect("Cargo.toml needs a parent")
        .to_path_buf();
    fs_extra::copy_items(&vec!["./nixpkgs.nix"], &project_dir, &CopyOptions::new())
        .expect("while copying nixpkgs.nix");

    // Get metadata
    eprintln!("Generating build file:");
    let before_build = Instant::now();

    let default_nix_path = cargo_toml.parent().unwrap().join("default.nix");
    let metadata = BuildInfo::for_config(
        &GenerateInfo::default(),
        &GenerateConfig {
            cargo_toml: cargo_toml.clone(),
            output: default_nix_path.clone(),
            nixpkgs_path: "./nixpkgs.nix".to_string(),
            crate_hashes_json: project_dir.join("crate-hashes.json").to_path_buf(),
        },
    )
    .unwrap();
    let default_nix_content = render::render_build_file(&metadata).unwrap();

    eprintln!(
        "Generating build file: done, took {} ms.",
        before_build.elapsed().as_millis()
    );

    eprintln!("Copy changed Cargo.lock/crate-hashes.json back to source: ");

    // Generate nix file
    render::write_to_file(&default_nix_path, &default_nix_content).unwrap();

    if default_nix_content.contains(".cargo")
        || default_nix_content.contains("registry/src/github.com")
        || default_nix_content.contains("/home/")
    {
        dump_with_lines(&default_nix_path).unwrap();
        panic!("Build file contained forbidden strings. Probably referencing .cargo directories.");
    }

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

    eprintln!("Copy changed Cargo.lock/crate-hashes.json back to source: done.");

    eprintln!("Calling nix build: ");
    nix_build(&project_dir, nix_attr, features).unwrap();
    eprintln!("Calling nix build: done.");

    eprintln!("Executing resulting binary: ");
    let bin_path = project_dir.join("result").join("bin").join(binary_name);
    let output = run_cmd(bin_path).unwrap();
    eprintln!("Executing resulting binary: done.");

    temp_dir.close().expect("couldn't remove temp dir");
    output
}

#[test]
fn clean_output_without_dot() {
    generate("Cargo.nix");
}

fn generate(path: &str) {
    let metadata = BuildInfo::for_config(
        &GenerateInfo {
            crate2nix_arguments: vec!["generate", "-n", "./nixpkgs.nix", "-o", path]
                .iter()
                .map(std::string::ToString::to_string)
                .collect(),
            ..GenerateInfo::default()
        },
        &GenerateConfig {
            cargo_toml: PathBuf::from("./Cargo.toml"),
            output: PathBuf::from(path),
            nixpkgs_path: "./nixpkgs.nix".to_string(),
            crate_hashes_json: PathBuf::from("./crate-hashes.json"),
        },
    )
    .unwrap();
    let rerendered_default_nix = render::render_build_file(&metadata).unwrap();

    if rerendered_default_nix.contains(" /home/") || rerendered_default_nix.contains(".cargo") {
        dump_with_lines("./Cargo.nix").unwrap();
        panic!("Build file contains forbidden strings.");
    }
}
