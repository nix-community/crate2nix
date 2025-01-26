//! Homeless code. Usually abstract and algorithmic.

use core::{convert::AsRef, fmt::Display};
use std::collections::BTreeSet;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

/// Return all occurrences of each item after the first.
/// ```
/// use crate2nix::util::find_duplicates;
/// assert_eq!(find_duplicates(Vec::<&str>::new().iter()), Vec::<&&str>::new());
/// assert_eq!(find_duplicates(vec!["a", "b"].iter()), Vec::<&&str>::new());
/// assert_eq!(find_duplicates(vec!["a", "b", "a"].iter()), vec![&"a"]);
/// assert_eq!(find_duplicates(vec!["a", "b", "a", "b", "a"].iter()), vec![&"a", &"b", &"a"]);
/// ```
pub fn find_duplicates<'a, T: Ord>(source: impl Iterator<Item = &'a T>) -> Vec<&'a T> {
    let mut seen = BTreeSet::new();
    source.filter(|v| !seen.insert(*v)).collect()
}

/// Newtype for a string that has been verified to be a git commit hash, and has been normalized.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Deserialize, Serialize)]
#[serde(try_from = "String")]
pub struct CommitHash(String);

impl CommitHash {
    /// If the string contains 40 hexadecimal characters returns a normalized string by trimming
    /// leading and trailing whitespace, and converting alphabetical characters to lower case.
    pub fn parse(input: &str) -> Option<Self> {
        let normalized = input.trim().to_lowercase();
        if normalized.len() == 40 && normalized.chars().all(|c| c.is_ascii_hexdigit()) {
            Some(CommitHash(normalized))
        } else {
            None
        }
    }
}

impl AsRef<str> for CommitHash {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for CommitHash {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        <Self as TryFrom<&str>>::try_from(&value)
    }
}

impl TryFrom<&str> for CommitHash {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        CommitHash::parse(value).ok_or_else(|| anyhow!("value {value} is not a git commit hash"))
    }
}

impl Display for CommitHash {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}
