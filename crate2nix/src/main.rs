use quicli::prelude::CliResult;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

use crate2nix::render;
use failure::format_err;
use failure::{bail, Error};
use serde::Deserialize;
use serde::Serialize;
use std::str::FromStr;

const DEFAULT_OUTPUT: &str = "./Cargo.nix";

#[derive(Debug, StructOpt, Deserialize, Serialize)]
#[structopt(
    name = "crate2nix",
    about = "Nix build file generator for a cargo rust projects."
)]
pub enum Opt {
    #[structopt(
        name = "generate",
        about = "Generate a nix file from a cargo rust project."
    )]
    Generate {
        #[structopt(
            short = "f",
            long = "cargo-toml",
            parse(from_os_str),
            help = "The path to the Cargo.toml of the project.",
            default_value = "./Cargo.toml"
        )]
        cargo_toml: PathBuf,

        #[structopt(
            long = "all-features",
            help = "Resolve project dependencies with all features enabled. \
                    This is the default and does not need to be specified. \
                    Users can choose their sub set of features and evaluation time so \
                    that one generated build file can be used for different feature selections."
        )]
        all_features: bool,

        #[structopt(
            long = "default-features",
            help = "Enables the default default features \
                    (instead of all features as is the default). \
                    Often combined with --features to add selected features on top."
        )]
        default_features: bool,

        #[structopt(
            long = "no-default-features",
            help = "Disables all features. \
                    Often combined with --features to reenable selected features."
        )]
        no_default_features: bool,

        #[structopt(
            long = "features",
            help = "Resolve project dependencies additionally with these features enabled. \
                    By default, all features are resolved."
        )]
        features: Vec<String>,

        #[structopt(
            short = "o",
            long = "output",
            help = "The path of the output.nix file. Uses ./Cargo.nix by default."
        )]
        output: Option<PathBuf>,

        #[structopt(
            short = "n",
            long = "nixpkgs-path",
            help = "The default path for the nixpkgs to use.",
            default_value = "<nixpkgs>"
        )]
        nixpkgs_path: String,

        #[structopt(
            short = "h",
            long = "crate-hashes",
            parse(from_os_str),
            help = "The path to the crate hash cache file. \
                    Uses 'crate-hashes.json' in the same directory as Cargo.toml by default."
        )]
        crate_hashes: Option<PathBuf>,

        // Mostly useful for testing
        #[structopt(
            long = "no-cargo-lock-checksums",
            help = "(FOR TESTING) Do not use checksums from Cargo.lock."
        )]
        no_cargo_lock_checksums: bool,

        #[structopt(
            long = "dont-read-crate-hashes",
            help = "(FOR TESTING) Do not read crate-hashes file. \
                    If there are any prefetches, their hashes will still be written into crate-hashes.json."
        )]
        dont_read_crate_hashes: bool,
    },

    #[structopt(
        name = "completions",
        about = "Generate auto-completions for the shell."
    )]
    Completions {
        #[structopt(
            short = "s",
            long = "shell",
            parse(from_str),
            help = "The shell to generate completions for. Specify 'invalid' to get a list of possibilities.",
            default_value = "bash"
        )]
        shell: String,

        #[structopt(
            short = "o",
            long = "output",
            help = "The path of the output directory.",
            default_value = "."
        )]
        output: PathBuf,
    },
}

fn main() -> CliResult {
    let opt = Opt::from_args();
    match opt {
        Opt::Generate {
            cargo_toml,
            output: opt_output,
            nixpkgs_path,
            crate_hashes,
            all_features,
            default_features,
            no_default_features,
            features,
            no_cargo_lock_checksums,
            dont_read_crate_hashes,
        } => {
            let crate_hashes_json = crate_hashes.unwrap_or_else(|| {
                cargo_toml
                    .parent()
                    .expect("Cargo.toml has parent")
                    .join("crate-hashes.json")
            });

            let generate_info = crate2nix::GenerateInfo::default();
            let output: PathBuf = opt_output
                .map(|v| Ok(v) as Result<_, Error>)
                .unwrap_or_else(|| {
                    if Path::new("DEFAULT_OUTPUT").exists() {
                        return Err(format_err!(
                            "No explicit output given and {} already exists.",
                            DEFAULT_OUTPUT
                        ));
                    }
                    Ok(DEFAULT_OUTPUT.into())
                })?;

            let feature_metadata_options = || {
                let mut options = Vec::new();

                if [all_features, default_features, no_default_features]
                    .iter()
                    .filter(|x| **x)
                    .count()
                    > 1
                {
                    bail!(
                        "Please specify at most one of \
                         --all-features, --no-default-features and --default-features."
                    )
                }

                // "cargo metadata" will default to the "default features".
                // crate2nix defaults to "--all-features" since this allows users to choose
                // any set of features at evaluation time.
                let all_features = !no_default_features && !default_features;
                if no_default_features {
                    options.push("--no-default-features".to_string());
                } else if !default_features {
                    assert!(all_features);
                    options.push("--all-features".to_string());
                }

                if !features.is_empty() {
                    if all_features {
                        bail!(
                            "You specified --features but --all-features was already selected. \
                               Use --no-default-features or --default-features to only select \
                               some features as a basis and then use --features to add additional \
                               features on top."
                        )
                    }
                    options.push("--features".to_string());
                    options.push(features.join(" "));
                }

                Ok(options)
            };

            let generate_config = crate2nix::GenerateConfig {
                cargo_toml,
                output: output.clone(),
                nixpkgs_path,
                crate_hashes_json,
                other_metadata_options: feature_metadata_options()?,
                use_cargo_lock_checksums: !no_cargo_lock_checksums,
                read_crate_hashes: !dont_read_crate_hashes,
            };
            let build_info = crate2nix::BuildInfo::for_config(&generate_info, &generate_config)?;
            render::CARGO_NIX.write_to_file(&output, &build_info)?;
        }
        Opt::Completions { shell, output } => {
            let shell = FromStr::from_str(&shell).map_err(|s| format_err!("{}", s))?;
            Opt::clap().gen_completions(env!("CARGO_PKG_NAME"), shell, output);
        }
    }

    Ok(())
}
