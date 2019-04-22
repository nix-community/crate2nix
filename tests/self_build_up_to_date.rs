use crate2nix::{nix_build::dump_with_lines, render, BuildInfo, GenerateConfig, GenerateInfo};
use std::path::PathBuf;

#[test]
fn up_to_date() {
    let metadata = BuildInfo::for_config(
        &GenerateInfo {
            crate2nix_arguments: vec!["generate", "-n", "<nixos-unstable>", "-o", "./Cargo.nix"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            ..GenerateInfo::new()
        },
        &GenerateConfig {
            cargo_toml: PathBuf::from("./Cargo.toml"),
            output: PathBuf::from("./Cargo.nix"),
            nixpkgs_path: "<nixos-unstable>".to_string(),
            crate_hashes_json: PathBuf::from("./crate-hashes.json"),
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
