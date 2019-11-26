use quicli::prelude::CliResult;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

use crate2nix::render;
use failure::format_err;
use failure::Error;
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
            help = "The path to the crate hash cache file. Uses 'crate-hashes.json' in the same directory as Cargo.toml by default."
        )]
        crate_hashes: Option<PathBuf>,
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
                        )
                        .into());
                    }
                    Ok(DEFAULT_OUTPUT.into())
                })?;
            let generate_config = crate2nix::GenerateConfig {
                cargo_toml,
                output: output.clone(),
                nixpkgs_path,
                crate_hashes_json,
            };
            let build_info = crate2nix::BuildInfo::for_config(&generate_info, &generate_config)?;
            let nix_string = render::render_build_file(&build_info)?;
            render::write_to_file(&output, &nix_string)?;
        }
        Opt::Completions { shell, output } => {
            let shell = FromStr::from_str(&shell).map_err(|s| format_err!("{}", s))?;
            Opt::clap().gen_completions(env!("CARGO_PKG_NAME"), shell, output);
        }
    }

    Ok(())
}
