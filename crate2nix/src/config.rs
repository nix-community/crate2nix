//! Managing the `crate2nix.json` config.

use anyhow::{bail, Context, Error};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fmt::Display,
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
    str::FromStr,
};

impl Config {
    /// Read config from path.
    pub fn read_from_or_default(path: &Path) -> Result<Config, Error> {
        if !path.exists() {
            return Ok(Config::default());
        }

        let file = File::open(path).context(format!("while opening {}", path.to_string_lossy()))?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).context(format!(
            "while deserializing config: {}",
            path.to_string_lossy()
        ))
    }

    /// Write config to path.
    pub fn write_to(&self, path: &Path) -> Result<(), Error> {
        let file =
            File::create(path).context(format!("while opening {}", path.to_string_lossy()))?;
        let writer = BufWriter::new(file);
        Ok(serde_json::to_writer_pretty(writer, self)?)
    }
}

/// The `crate2nix.json` config data.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    /// Out of tree sources.
    pub sources: BTreeMap<String, Source>,
}

impl Config {
    /// Add or replace a source. Returns the old source if there was one.
    pub fn upsert_source(
        &mut self,
        explicit_name: Option<String>,
        source: Source,
    ) -> Option<Source> {
        let name = explicit_name
            .or_else(|| source.name().map(|s| s.to_string()))
            .expect("No name given");
        self.sources.insert(name, source)
    }

    /// Prints all sources to stdout.
    pub fn print_sources(&self) {
        if self.sources.is_empty() {
            eprintln!("No sources configured.\n");
            return;
        }

        let max_len = self
            .sources
            .iter()
            .map(|(n, _)| n.len())
            .max()
            .unwrap_or_default();
        for (name, source) in &self.sources {
            println!("{:width$} {}", name, source, width = max_len);
            println!();
            println!(
                "{:width$} crate2nix source add {}",
                "",
                source.as_command(name),
                width = max_len
            );
            println!();
        }
    }
}

/// An out of tree source.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum Source {
    /// Get the source from crates.io.
    CratesIo {
        /// The crate name.
        name: String,
        /// The exact crate version to fetch.
        version: semver::Version,
        /// The sha256 hash of the source.
        sha256: String,
    },
    /// Get the source from git.
    Git {
        /// The URL of the git repository.
        ///
        /// E.g. https://github.com/kolloch/crate2nix.git
        #[serde(with = "url_serde")]
        url: url::Url,
        /// The revision hash.
        rev: String,
        /// The sha256 of the fetched result.
        sha256: String,
    },
    /// Get the source from a nix expression.
    Nix {
        /// The nixfile to include.
        #[serde(flatten)]
        file: NixFile,
        /// A Nix attribute path which will be resolved against the file.
        #[serde(skip_serializing_if = "Option::is_none")]
        attr: Option<String>,
    },
    /// The source is already obtained, useful for IFD.
    LocalDirectory {
        /// Path to the source.
        path: PathBuf,
    },
}

/// A nix file path which is either included by `import` or `callPackage`.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Eq, Hash)]
pub enum NixFile {
    /// A file path that should be imported.
    #[serde(rename = "import")]
    Import(String),
    /// A file path the should be included by `pkgs.callPackage`.
    #[serde(rename = "package")]
    Package(String),
}

impl Display for NixFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Import(path) => write!(f, "import {}", path),
            Self::Package(path) => write!(f, "pkgs.callPackage {} {{}}", path),
        }
    }
}

impl NixFile {
    /// Returns the chosen file option as CLI string.
    pub fn as_command(&self) -> String {
        match self {
            Self::Import(path) => format!("--import '{}'", path),
            Self::Package(path) => format!("--package '{}'", path),
        }
    }
}

impl Source {
    /// The name of the source.
    pub fn name(&self) -> Option<&str> {
        match self {
            Source::CratesIo { name, .. } => Some(name),
            Source::Git { url, .. } => {
                let path = url.path();
                let after_last_slash = path.split('/').last().unwrap_or(path);
                let without_dot_git = after_last_slash
                    .strip_suffix(".git")
                    .unwrap_or(after_last_slash);
                Some(without_dot_git)
            }
            Source::Nix {
                attr: Some(attr), ..
            } => attr.split('.').last().or(if attr.trim().is_empty() {
                None
            } else {
                Some(attr.trim())
            }),
            _ => None,
        }
    }
}

impl Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Source::CratesIo {
                name,
                version,
                sha256,
            } => write!(f, "{} {} from crates.io: {}", name, version, sha256),
            Source::Git { url, rev, sha256 } => write!(f, "{}#{} via git: {}", url, rev, sha256),
            Source::Nix { file, attr: None } => write!(f, "{}", file),
            Source::Nix {
                file,
                attr: Some(attr),
            } => write!(f, "({}).{}", file, attr),
            Source::LocalDirectory { path } => write!(f, "{}", path.display()),
        }
    }
}

impl Source {
    /// Returns a CLI string to reproduce this source.
    pub fn as_command(&self, name: &str) -> String {
        match self {
            Source::CratesIo {
                name: crate_name,
                version,
                ..
            } => format!("cratesIo --name '{}' '{}' '{}'", name, crate_name, version),
            Source::Git { url, rev, .. } => {
                format!("git --name '{}' '{}' --rev {}", name, url, rev)
            }
            Source::Nix { file, attr: None } => {
                format!("nix --name '{}' {}", name, file.as_command())
            }
            Source::Nix {
                file,
                attr: Some(attr),
            } => format!("nix --name '{}' {} '{}'", name, file.as_command(), attr),
            Source::LocalDirectory { path } => {
                format!("path --name '{}' {}", name, path.display())
            },
        }
    }
}

/// The type of a Source.
#[derive(Debug, Serialize, Deserialize)]
pub enum SourceType {
    /// Corresponds to Source::CratesIo.
    CratesIo,
    /// Corresponds to Source::Git.
    Git,
}

impl FromStr for SourceType {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cratesIo" => Ok(SourceType::CratesIo),
            "git" => Ok(SourceType::Git),
            _ => bail!("unkown source type: {}", s),
        }
    }
}
