//! Homeless code. Usually abstract and algorithmic.

use std::collections::BTreeSet;

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
