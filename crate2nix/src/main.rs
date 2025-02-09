use std::path::{Path, PathBuf};
use structopt::clap::ArgGroup;
use structopt::StructOpt;

use anyhow::format_err;
use anyhow::{bail, Error};
use crate2nix::{
    config::{Config, NixFile},
    normalize_manifest::normalize_manifest,
    render, CommitHash,
};
use semver::Version;
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
        about = "Generates a Cargo.nix file from a cargo rust project."
    )]
    Generate {
        #[structopt(
            short = "c",
            long = "config",
            parse(from_os_str),
            help = "The path to the crate2nix.json file (same directory as Cargo.nix ...).",
            default_value = "./crate2nix.json"
        )]
        crate2nix_json: PathBuf,

        #[structopt(
            short = "f",
            long = "cargo-toml",
            parse(from_os_str),
            help = "The path to the Cargo.toml of the project."
        )]
        cargo_toml: Vec<PathBuf>,

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
                    Uses 'crate-hashes.json' in the same directory as the Cargo.nix output by default."
        )]
        crate_hashes: Option<PathBuf>,

        #[structopt(
            short = "r",
            long = "registry-hashes",
            parse(from_os_str),
            help = "The path to the registry hash cache file. \
                    Uses 'registry-hashes.json' in the same directory as the Cargo.nix output by default."
        )]
        registry_hashes: Option<PathBuf>,

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

    #[structopt(name = "source", about = "Manage out of tree sources for crate2nix.")]
    Source {
        #[structopt(
            short = "c",
            long = "config",
            parse(from_os_str),
            help = "The path to the crate2nix.json file (same directory as Cargo.nix ...).",
            default_value = "./crate2nix.json"
        )]
        crate2nix_json: PathBuf,

        #[structopt(subcommand)]
        command: SourceCommands,
    },

    #[structopt(
        name = "completions",
        about = "Generates auto-completions for the shell."
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

    #[structopt(
        name = "normalize-manifest",
        about = "Resolve fields inherited from a workspace, so that the manifest can be processed stand-alone."
    )]
    NormalizeManifest {
        #[structopt(
            short = "f",
            long = "cargo-toml",
            parse(from_os_str),
            help = "The path to the Cargo.toml of the project."
        )]
        cargo_toml: PathBuf,
    },
}

#[derive(Debug, StructOpt, Deserialize, Serialize)]
#[structopt(about = "Support for managing out-of-tree sources.")]
pub enum SourceCommands {
    #[structopt(name = "add", about = "Adds source, prefetching it if when necessary.")]
    Add {
        #[structopt(subcommand)]
        command: SourceAddingCommands,
    },
    #[structopt(name = "remove", about = "Removes source.")]
    Remove {
        #[structopt(long = "name", help = "The name of the source to remove.")]
        name: String,
    },
    #[structopt(name = "list", about = "Lists all sources.")]
    List,

    #[structopt(
        name = "fetch",
        about = "Fetch all sources with nix.\n\
                 This is usually called automatically and mostly useful for testing."
    )]
    Fetch,
    #[structopt(
        name = "generate",
        about = "Generate crate2nix-sources.nix.\n\
                 This is usually called automatically and mostly useful for testing."
    )]
    Generate,
}

impl SourceCommands {
    pub fn execute(self, crate2nix_json: &Path) -> Result<(), Error> {
        match self {
            SourceCommands::Add { command, .. } => command.execute(crate2nix_json),
            SourceCommands::List => {
                let config = Config::read_from_or_default(crate2nix_json)?;
                config.print_sources();
                Ok(())
            }
            SourceCommands::Remove { name } => {
                let mut config = Config::read_from_or_default(crate2nix_json)?;
                if config.sources.is_empty() {
                    eprintln!(
                        "No sources configured in {}.",
                        crate2nix_json.to_string_lossy()
                    );
                } else {
                    let removed = config.sources.remove(&name);
                    if let Some(removed) = removed {
                        config.write_to(crate2nix_json)?;
                        eprintln!("Removed source\n\t{}", removed);
                    } else {
                        eprintln!("Source '{}' not found among the following sources.\n", name);
                        config.print_sources();
                    }
                }
                Ok(())
            }
            SourceCommands::Fetch => {
                let sources = crate2nix::sources::FetchedSources::new(crate2nix_json);
                let output = sources.fetch()?;
                println!("Fetched sources into {}", output.to_string_lossy());
                Ok(())
            }
            SourceCommands::Generate => {
                let sources = crate2nix::sources::FetchedSources::new(crate2nix_json);
                sources.regenerate_sources_nix()
            }
        }
    }
}

#[derive(Debug, StructOpt, Deserialize, Serialize)]
pub enum SourceAddingCommands {
    #[structopt(name = "cratesIo", about = "Adds source from crates.io.")]
    CratesIo {
        #[structopt(
            long = "name",
            help = "Use this source name instead of the crate name.\n\
                    The source name is used as a workspaceMember name."
        )]
        name: Option<String>,

        #[structopt(help = "The crate name on crates.io.")]
        crate_name: String,

        #[structopt(help = "The full version of the crate.")]
        crate_version: Version,
    },

    #[structopt(
        name = "git",
        about = "Adds git source.\n\
                 \n\
                 If you want auto-update support, consider using the \"nix\" source type\n\
                 and manage the sources with niv.\n\
                 \n\
                 See https://github.com/nmattia/niv."
    )]
    Git {
        #[structopt(
            long = "name",
            help = "Use this source name instead of the last URL path segment without '.git'.\n\
                    The source name is used as a workspaceMember name."
        )]
        name: Option<String>,

        /// The URL of the git repository.
        ///
        /// E.g. https://github.com/kolloch/crate2nix.git
        url: url::Url,

        #[structopt(long = "rev", parse(try_from_str = CommitHash::try_from), help = "The git revision hash.")]
        rev: CommitHash,
    },

    #[structopt(
        name = "nix",
        about = "Adds nix attribute from a file as source.\n\
                E.g. crate2nix source add --import nix/sources.nix my_crate.
                 \n\
                 This is the most flexible source type.\n\
                 Works well with tools like niv which support easy updating.",
        // We need either an `--import` or a `--package`.
        group = ArgGroup::with_name("file").required(true),
        // We need an explicit `--name` or an `attr` to derive the name from.
        group = ArgGroup::with_name("some_name").multiple(true).required(true),
    )]
    Nix {
        #[structopt(
            long,
            help = "The name of this source \n\
                    if you do not want to use the last element of the attribute path.",
            group = "some_name"
        )]
        name: Option<String>,

        #[structopt(long, group = "file", help = "A path to `import` in nix.")]
        import: Option<String>,

        #[structopt(
            long,
            group = "file",
            help = "A path to call with `pkgs.callPackage` in nix."
        )]
        package: Option<String>,

        #[structopt(
            help = "The attribute path that leads to the source derivation.",
            group = "some_name"
        )]
        attr: Option<String>,
    },
}

impl SourceAddingCommands {
    pub fn execute(self, crate2nix_json: &Path) -> Result<(), Error> {
        let (name, source) = match self {
            SourceAddingCommands::CratesIo {
                name,
                crate_name,
                crate_version,
            } => {
                let source = crate2nix::sources::crates_io_source(crate_name, crate_version)?;
                (name, source)
            }
            SourceAddingCommands::Git { name, url, rev } => {
                let source = crate2nix::sources::git_io_source(url, rev)?;
                (name, source)
            }
            SourceAddingCommands::Nix {
                name,
                import,
                package,
                attr,
            } => {
                let file = match (import, package) {
                    (Some(import), _) => NixFile::Import(import),
                    (_, Some(package)) => NixFile::Package(package),
                    _ => unreachable!("no file argument given"),
                };

                (name, crate2nix::config::Source::Nix { file, attr })
            }
        };
        let mut config = Config::read_from_or_default(crate2nix_json)?;
        let old_source = config.upsert_source(name, source.clone());
        config.write_to(crate2nix_json)?;
        match old_source {
            Some(old_source) => {
                eprintln!(
                    "Updated existing source\n\t{}\nto\n\t{}",
                    old_source, source
                );
            }
            None => {
                eprintln!("Added new source: {}", source);
            }
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let opt = Opt::from_args();
    match opt {
        Opt::Generate {
            crate2nix_json,
            mut cargo_toml,
            output: opt_output,
            nixpkgs_path,
            crate_hashes,
            registry_hashes,
            all_features,
            default_features,
            no_default_features,
            features,
            no_cargo_lock_checksums,
            dont_read_crate_hashes,
        } => {
            let config = crate2nix::config::Config::read_from_or_default(&crate2nix_json)?;

            if !config.sources.is_empty() {
                let fetched_sources = crate2nix::sources::FetchedSources::new(&crate2nix_json);
                let cargo_tomls = fetched_sources.get_cargo_tomls()?;
                cargo_toml.extend(cargo_tomls);
            }

            if cargo_toml.is_empty() {
                cargo_toml.push("./Cargo.toml".into());
            }

            let output: PathBuf = opt_output
                .map(|v| Ok(v) as Result<_, Error>)
                .unwrap_or_else(|| {
                    crate2nix::render::check_generated_by_crate2nix(DEFAULT_OUTPUT)?;
                    Ok(DEFAULT_OUTPUT.into())
                })?;

            let crate_hashes_json = crate_hashes.unwrap_or_else(|| {
                output
                    .parent()
                    .expect("Cargo.nix has parent")
                    .join("crate-hashes.json")
            });

            let registry_hashes_json = registry_hashes.unwrap_or_else(|| {
                output
                    .parent()
                    .expect("Cargo.nix has parent")
                    .join("registry-hashes.json")
            });

            let generate_info = crate2nix::GenerateInfo::default();

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
                registry_hashes_json,
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
        Opt::Source {
            crate2nix_json,
            command,
        } => {
            command.execute(&crate2nix_json)?;
        }
        Opt::NormalizeManifest { cargo_toml } => {
            let manifest = normalize_manifest(&cargo_toml)?;
            let toml = toml::to_string_pretty(&manifest)?;
            println!("{toml}");
        }
    }

    Ok(())
}
