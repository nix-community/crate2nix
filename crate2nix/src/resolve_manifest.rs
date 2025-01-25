//! Expand a Cargo.toml manifest to include any data inherited from its workspace.

use std::path::Path;

use anyhow::Context as _;
use cargo_toml::Manifest;

/// Loads a Cargo.toml manifest (which may be a workspace member manifest) from the given path.
/// Automatically locates the corresponding manifest if there is one to fill in inherited values.
///
/// For example `version.workspace = true` becomes `version = "1.2.3"`
pub fn resolve_manifest(cargo_toml: &Path) -> Result<toml::Value, anyhow::Error> {
    // Expands most, but not all, inherited workspace values. See note below.
    let manifest = Manifest::from_path(cargo_toml)?;

    // As of this comment cargo_toml does not expand inherited lints. For example manifest content
    // like this:
    //
    //     [lints]
    //     workspace = true
    //
    // is left as-is. The presence of a `workspace = true` setting leads to a read error from
    // `cargo metadata` later on. To avoid this and similar issues this fixup step walks through
    // the normalized manifest's TOML nodes to remove any table entries of `workspace = true`.

    let toml = toml::Value::try_from(manifest).with_context(|| {
        format!(
            "error converting manifest at {} back to TOML after normalization",
            cargo_toml.to_string_lossy()
        )
    })?;

    let toml =
        prune_workspace_references(toml).unwrap_or(toml::Value::Table(toml::map::Map::new()));

    Ok(toml)
}

fn prune_workspace_references(toml: toml::Value) -> Option<toml::Value> {
    match toml {
        toml::Value::Table(map) => {
            let orig_is_empty = map.is_empty();
            let pruned = map
                .into_iter()
                .filter(|entry| !is_workspace_reference(entry))
                .filter_map(|(key, value)| Some((key, prune_workspace_references(value)?)))
                .collect::<toml::map::Map<_, _>>();
            if pruned.is_empty() && !orig_is_empty {
                None
            } else {
                Some(toml::Value::Table(pruned))
            }
        }
        toml::Value::Array(vec) => Some(toml::Value::Array(
            vec.into_iter()
                .filter_map(prune_workspace_references)
                .collect(),
        )),
        value => Some(value),
    }
}

fn is_workspace_reference((key, value): &(String, toml::Value)) -> bool {
    key == "workspace" && value == &toml::Value::Boolean(true)
}
