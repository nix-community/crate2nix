//! Code for extracting hashes and more from Cargo.lock

use anyhow::{format_err, Error};
use cargo_metadata::PackageId;
use serde::{de, ser, Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::path::Path;
use std::str::FromStr;

impl EncodableResolve {
    pub fn load_lock_file(path: &Path) -> Result<EncodableResolve, Error> {
        let config = &std::fs::read_to_string(path)
            .map_err(|e| format_err!("while reading lock file {}: {}", path.display(), e))?;
        Self::load_lock_string(path, config)
    }

    pub fn load_lock_string(path: &Path, config: &str) -> Result<EncodableResolve, Error> {
        let resolve: toml::Value = toml::from_str(config)
            .map_err(|e| format_err!("while parsing toml from {}: {}", path.display(), e))?;

        let v: EncodableResolve = resolve
            .try_into()
            .map_err(|e| format_err!("unexpected format in {}: {}", path.display(), e))?;
        Ok(v)
    }

    pub fn get_hashes_by_package_id(
        &self,
        hashes: &mut HashMap<PackageId, String>,
    ) -> Result<(), Error> {
        for EncodableDependency {
            name,
            version,
            source,
            checksum,
            ..
        } in self.package.iter()
        {
            if let (Some(source), Some(checksum)) = (source, checksum) {
                let package_id = PackageId {
                    repr: format!("{} {} ({})", name, version, source),
                };
                if checksum != "<none>" {
                    hashes.insert(package_id, checksum.clone());
                }
            }
        }

        // Retrieve legacy checksums.
        const CHECKSUM_PREFIX: &str = "checksum ";
        if let Some(metadata) = &self.metadata {
            for (key, value) in metadata {
                if key.starts_with(CHECKSUM_PREFIX) {
                    let package_id = PackageId {
                        repr: key.trim_start_matches(CHECKSUM_PREFIX).to_string(),
                    };
                    if value != "<none>" {
                        hashes.insert(package_id, value.clone());
                    }
                }
            }
        }

        Ok(())
    }
}

#[test]
fn test_no_legacy_checksums() {
    let config = r#"
[[package]]
name = "aho-corasick"
version = "0.7.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
        dependencies = [
        "memchr 2.3.0 (registry+https://github.com/rust-lang/crates.io-index)",
    ]
"#;
    let resolve = EncodableResolve::load_lock_string(Path::new("dummy"), config).unwrap();
    let mut hashes = HashMap::new();
    resolve.get_hashes_by_package_id(&mut hashes).unwrap();
    assert_eq!(hashes, HashMap::new());
}

#[test]
fn test_some_legacy_checksums() {
    let config = r#"
[[package]]
name = "aho-corasick"
version = "0.7.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
dependencies = [
  "memchr 2.3.0 (registry+https://github.com/rust-lang/crates.io-index)",
]

[metadata]
"checksum structopt 0.2.18 (registry+https://github.com/rust-lang/crates.io-index)" = "16c2cdbf9cc375f15d1b4141bc48aeef444806655cd0e904207edc8d68d86ed7"
"checksum structopt-derive 0.2.18 (registry+https://github.com/rust-lang/crates.io-index)" = "53010261a84b37689f9ed7d395165029f9cc7abb9f56bbfe86bee2597ed25107"

"#;
    let resolve = EncodableResolve::load_lock_string(Path::new("dummy"), config).unwrap();
    let mut hashes = HashMap::new();
    resolve.get_hashes_by_package_id(&mut hashes).unwrap();
    assert_eq!(
        hashes,
        [(
                PackageId { repr: "structopt 0.2.18 (registry+https://github.com/rust-lang/crates.io-index)".to_string() },
                "16c2cdbf9cc375f15d1b4141bc48aeef444806655cd0e904207edc8d68d86ed7"
            ),
            (
                PackageId { repr: "structopt-derive 0.2.18 (registry+https://github.com/rust-lang/crates.io-index)".to_string()},
                "53010261a84b37689f9ed7d395165029f9cc7abb9f56bbfe86bee2597ed25107"
            )]
        .iter()
        .map(|(package_id, hash)| (package_id.clone(), hash.to_string()))
        .collect::<HashMap<_, _>>()
    );
}

#[test]
fn test_some_inline_checksums() {
    let config = r#"
[[package]]
name = "aho-corasick"
version = "0.7.6"
source = "registry+https://github.com/rust-lang/crates.io-index"
dependencies = [
  "memchr 2.3.0 (registry+https://github.com/rust-lang/crates.io-index)",
]
checksum = "16c2cdbf9cc375f15d1b4141bc48aeef444806655cd0e904207edc8d68d86ed7"
"#;
    let resolve = EncodableResolve::load_lock_string(Path::new("dummy"), config).unwrap();
    let mut hashes = HashMap::new();
    resolve.get_hashes_by_package_id(&mut hashes).unwrap();
    assert_eq!(
        hashes,
        [(
            PackageId {
                repr: "aho-corasick 0.7.6 (registry+https://github.com/rust-lang/crates.io-index)"
                    .to_string()
            },
            "16c2cdbf9cc375f15d1b4141bc48aeef444806655cd0e904207edc8d68d86ed7"
        )]
        .iter()
        .map(|(package_id, hash)| (package_id.clone(), hash.to_string()))
        .collect::<HashMap<_, _>>()
    );
}

//
// The code below was copied/adjusted from Cargo.
//

/// The `Cargo.lock` structure.
#[derive(Serialize, Deserialize, Debug)]
pub struct EncodableResolve {
    package: Vec<EncodableDependency>,
    /// `root` is optional to allow backward compatibility.
    root: Option<EncodableDependency>,
    metadata: Option<Metadata>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Patch {
    unused: Vec<EncodableDependency>,
}

pub type Metadata = BTreeMap<String, String>;

#[derive(Serialize, Deserialize, Debug, PartialOrd, Ord, PartialEq, Eq)]
pub struct EncodableDependency {
    name: String,
    version: String,
    source: Option<String>,
    checksum: Option<String>,
    dependencies: Option<Vec<EncodablePackageId>>,
    replace: Option<EncodablePackageId>,
}

#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Clone)]
pub struct EncodablePackageId {
    name: String,
    version: Option<String>,
    source: Option<String>,
}

impl fmt::Display for EncodablePackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if let Some(s) = &self.version {
            write!(f, " {}", s)?;
        }
        if let Some(s) = &self.source {
            write!(f, " ({})", s)?;
        }
        Ok(())
    }
}

impl FromStr for EncodablePackageId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<EncodablePackageId, Error> {
        let mut s = s.splitn(3, ' ');
        let name = s.next().unwrap();
        let version = s.next();
        let source_id = match s.next() {
            Some(s) => {
                if s.starts_with('(') && s.ends_with(')') {
                    Some(String::from(&s[1..s.len() - 1]))
                } else {
                    anyhow::bail!("invalid serialized PackageId")
                }
            }
            None => None,
        };

        Ok(EncodablePackageId {
            name: name.to_string(),
            version: version.map(|v| v.to_string()),
            source: source_id,
        })
    }
}

impl ser::Serialize for EncodablePackageId {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        s.collect_str(self)
    }
}

impl<'de> de::Deserialize<'de> for EncodablePackageId {
    fn deserialize<D>(d: D) -> Result<EncodablePackageId, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        String::deserialize(d).and_then(|string| {
            string
                .parse::<EncodablePackageId>()
                .map_err(de::Error::custom)
        })
    }
}
