use quicli::prelude::CliResult;
use std::path::PathBuf;
use structopt::StructOpt;

use cargo2nix::render;
use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Debug, StructOpt, Deserialize, Serialize)]
#[structopt(
    name = "cargo2nix",
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
            short = "n",
            long = "nixpkgs-path",
            help = "The default path for the nixpkgs to use.",
            default_value = "<nixos>"
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
}

fn main() -> CliResult {
    let opt = Opt::from_args();
    match opt {
        Opt::Generate {
            cargo_toml,
            nixpkgs_path,
            crate_hashes,
        } => {
            let crate_hashes_json = crate_hashes.unwrap_or_else(|| {
                cargo_toml
                    .parent()
                    .expect("Cargo.toml has parent")
                    .join("crate_hashes.json")
            });

            let generate_config = cargo2nix::GenerateConfig {
                cargo_toml,
                nixpkgs_path,
                crate_hashes_json,
            };
            let default_nix = cargo2nix::default_nix(&generate_config)?;
            println!("{}", render::default_nix(&default_nix)?);
            eprintln!("Nix build written completely.");
        }
    }

    Ok(())
}
