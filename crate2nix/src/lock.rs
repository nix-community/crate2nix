//! Code for extracting hashes and more from Cargo.lock

use anyhow::{format_err, Error};
use cargo_metadata::PackageId;
use serde::{de, ser, Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::path::Path;
use std::str::FromStr;

use crate::metadata::MergedMetadata;

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
        metadata: &MergedMetadata,
        hashes: &mut HashMap<PackageId, String>,
    ) -> Result<(), Error> {
        let mut package_id_by_source = HashMap::new();
        for p in &metadata.packages {
            let Some(ref source) = p.source else {
                // local crate
                continue;
            };
            let key = (p.name.as_str(), source.repr.as_str(), p.version.to_string());
            package_id_by_source.insert(key, &p.id);
        }

        for EncodableDependency {
            name,
            version,
            source,
            checksum,
            ..
        } in self.package.iter()
        {
            let Some(source) = source.as_ref() else {
                continue;
            };

            let Some(package_id) =
                package_id_by_source.get(&(name.as_str(), source.as_str(), version.clone()))
            else {
                continue;
            };

            let checksum = match checksum.as_ref() {
                Some(checksum) if checksum == "<none>" => None,
                Some(checksum) => Some(checksum),
                None => {
                    // Retrieve legacy checksums.
                    self.metadata.as_ref().and_then(|metadata| {
                        const CHECKSUM_PREFIX: &str = "checksum";
                        let checksum_key = format!("{CHECKSUM_PREFIX} {name} {version} ({source})");
                        metadata.get(&checksum_key)
                    })
                }
            };

            if let Some(checksum) = checksum {
                hashes.insert((*package_id).clone(), checksum.to_owned());
            }
        }

        Ok(())
    }
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
    metadata: Option<BTreeMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Patch {
    unused: Vec<EncodableDependency>,
}

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
