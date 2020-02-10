use colored_diff::PrettyDifference;
use crate2nix::{nix_build::dump_with_lines, render, BuildInfo, GenerateConfig, GenerateInfo};
use failure::{bail, format_err, Error};
use serde::Deserialize;
use serde::Serialize;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

#[test]
fn self_up_to_date() {
    let metadata = BuildInfo::for_config(
        &GenerateInfo {
            crate2nix_arguments: vec![
                "generate",
                "-n",
                "../nixpkgs.nix",
                "-f",
                "./crate2nix/Cargo.toml",
                "-o",
                "./crate2nix/Cargo.nix",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            ..GenerateInfo::default()
        },
        &GenerateConfig {
            cargo_toml: PathBuf::from("./Cargo.toml"),
            output: PathBuf::from("./Cargo.nix"),
            nixpkgs_path: "../nixpkgs.nix".to_string(),
            crate_hashes_json: PathBuf::from("./crate-hashes.json"),
            other_metadata_options: vec![],
            use_cargo_lock_checksums: true,
            read_crate_hashes: true,
        },
    )
    .unwrap();
    let rerendered_default_nix = render::render_build_file(&metadata).unwrap();
    let actual_default_nix = std::fs::read_to_string("./Cargo.nix").unwrap();
    assert_eq!(actual_default_nix, rerendered_default_nix);

    if rerendered_default_nix.contains(" /home/") || rerendered_default_nix.contains(".cargo") {
        dump_with_lines("./Cargo.nix").unwrap();
        panic!("Build file contains forbidden strings.");
    }
}

#[test]
fn pregenerated_up_to_date() {
    let test_configs = get_test_configs().expect("while running instantiate");
    // TODO: Regenerate build files and compare
    for test_config in test_configs {
        match test_config.pregenerated_build {
            Some(pregenerated_build) => {
                let cargo_nix = PathBuf::from_str(&pregenerated_build)
                    .expect("pregeneratedBuild must be valid path");
                assert_up_to_date(&cargo_nix.parent().expect("Cargo.nix must be in directory"));
            }
            None => println!("Skipping not pregenerated {}", test_config.name),
        }
    }
}

// Assert that the pregenerated build files are up to date, i.e.
// the current code would result in the same build file.
fn assert_up_to_date(project_dir: &Path) {
    let cargo_toml = project_dir.join("Cargo.toml");
    let output = project_dir.join("Cargo.nix");
    println!("Checking pregenerated {}", output.to_str().unwrap());
    let config = GenerateConfig {
        cargo_toml: PathBuf::from("../").join(cargo_toml.clone()),
        output: PathBuf::from("../").join(output.clone()),
        nixpkgs_path: "<nixpkgs>".to_string(),
        crate_hashes_json: PathBuf::from("../")
            .join(project_dir)
            .join("./crate-hashes.json"),
        other_metadata_options: vec![],
        use_cargo_lock_checksums: true,
        read_crate_hashes: true,
    };
    let metadata = BuildInfo::for_config(
        &GenerateInfo {
            crate2nix_arguments: vec![
                "generate",
                "-f",
                cargo_toml.to_str().unwrap(),
                "-o",
                output.to_str().unwrap(),
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            ..GenerateInfo::default()
        },
        &config,
    )
    .unwrap();
    let rerendered_default_nix = render::render_build_file(&metadata).unwrap();
    let actual_default_nix = std::fs::read_to_string(&config.output).unwrap();

    assert_eq!(
        actual_default_nix,
        rerendered_default_nix,
        "Pregenerated build files differ, please rerun ./regenerate_cargo_nix.sh.\n{}",
        PrettyDifference {
            actual: &actual_default_nix,
            expected: &rerendered_default_nix
        }
    );

    if rerendered_default_nix.contains(" /home/") || rerendered_default_nix.contains(".cargo") {
        dump_with_lines("./Cargo.nix").unwrap();
        panic!("Build file contains forbidden strings.");
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TestConfig {
    name: String,
    #[serde(rename = "pregeneratedBuild")]
    pregenerated_build: Option<String>,
}

fn get_test_configs() -> Result<Vec<TestConfig>, Error> {
    let output = Command::new("nix")
        .args(&["eval", "--json", "-f", "../tests.nix", "buildTestConfigs"])
        .output()
        .map_err(|e| format_err!("while spawning nix: {}", e))?;
    if !output.status.success() {
        std::io::stdout().write_all(&output.stdout)?;
        std::io::stderr().write_all(&output.stderr)?;
        bail!(
            "nix-instantiate\n=> exited with: {}",
            output.status.code().unwrap_or(-1)
        );
    }

    let json_string = String::from_utf8(output.stdout)
        .map_err(|_e| format_err!("output of nix-instantiate is not UTF8!"))?;

    Ok(serde_json::from_str(&json_string)?)
}
