use std::collections::BTreeSet;

/// Return all occurrences of each item after the first.
pub fn find_duplicates<'a, T: Ord>(source: impl Iterator<Item = &'a T>) -> Vec<&'a T> {
    let mut seen = BTreeSet::new();
    let mut duplicate = Vec::new();

    for v in source.into_iter() {
        if !seen.insert(v) {
            duplicate.push(v);
        }
    }

    duplicate
}
