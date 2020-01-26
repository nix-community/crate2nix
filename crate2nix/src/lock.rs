use failure::{format_err, Error};
use serde::{de, ser, Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;
use std::str::FromStr;

pub fn load_lock_file(path: &Path) -> Result<EncodableResolve, Error> {
    let resolve: toml::Value = toml::from_str(
        &std::fs::read_to_string(path)
            .map_err(|e| format_err!("while reading lock file {}: {}", path.display(), e))?,
    )
    .map_err(|e| format_err!("while parsing toml from {}: {}", path.display(), e))?;

    let v: EncodableResolve = resolve
        .try_into()
        .map_err(|e| format_err!("unexpected format in {}: {}", path.display(), e))?;
    Ok(v)
}

impl EncodableResolve {
    pub fn get_hash(&self, package_id_str: &str) -> Result<Option<String>, Error> {
        let key = format!("checksum {}", package_id_str);
        if let Some(hex_hash) = self
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get(&key))
        {
            let bytes = hex::decode(&hex_hash)?;
            return Ok(Some(nix_base32::to_nix_base32(&bytes)));
        }
        Ok(None)
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
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<EncodablePackageId, Error> {
        let mut s = s.splitn(3, ' ');
        let name = s.next().unwrap();
        let version = s.next();
        let source_id = match s.next() {
            Some(s) => {
                if s.starts_with('(') && s.ends_with(')') {
                    Some(String::from(&s[1..s.len() - 1]))
                } else {
                    failure::bail!("invalid serialized PackageId")
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
