//! Expand a Cargo.toml manifest to include any data inherited from its workspace.

use std::path::Path;

use anyhow::Context as _;
use cargo_toml::Manifest;

/// Loads a Cargo.toml manifest (which may be a workspace member manifest) from the given path.
/// Automatically locates the corresponding manifest if there is one to fill in inherited values.
///
/// For example `version.workspace = true` becomes `version = "1.2.3"`
pub fn normalize_manifest(cargo_toml: impl AsRef<Path>) -> Result<String, anyhow::Error> {
    // Expands most, but not all, inherited workspace values. See note below.
    let manifest = Manifest::from_path(&cargo_toml)?;

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
            cargo_toml.as_ref().to_string_lossy()
        )
    })?;

    let toml =
        prune_workspace_references(toml).unwrap_or(toml::Value::Table(toml::map::Map::new()));

    Ok(toml::to_string_pretty(&toml)?)
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

#[cfg(test)]
mod tests {
    use std::{
        env::temp_dir,
        fs::{create_dir_all, File},
        io::Write as _,
        path::PathBuf,
    };

    use super::normalize_manifest;

    #[test]
    fn normalizes_a_package_manifest_in_a_workspace() -> anyhow::Result<()> {
        let TestWorkspace {
            package_manifest, ..
        } = test_workspace()?;
        let normalized = normalize_manifest(&package_manifest)?;
        assert_eq!(
            normalized.trim(),
            r#"
[dependencies]
itertools = "^0.13.0"

[package]
edition = "2021"
name = "package"
version = "1.0.0"
"#
            .trim()
        );
        Ok(())
    }

    #[test]
    fn produces_consistent_output_for_normalized_manifest() -> anyhow::Result<()> {
        let TestWorkspace {
            package_manifest, ..
        } = test_workspace()?;
        let normalized = normalize_manifest(&package_manifest)?;
        for _ in 1..10 {
            let normalized_again = normalize_manifest(&package_manifest)?;
            assert_eq!(normalized_again, normalized);
        }

        Ok(())
    }

    #[derive(Debug)]
    struct TestWorkspace {
        package_manifest: PathBuf,
    }

    fn test_workspace() -> anyhow::Result<TestWorkspace> {
        let workspace_dir = temp_dir();

        let workspace_manifest = workspace_dir.join("Cargo.toml");
        File::create(workspace_manifest)?.write_all(
            br#"
[workspace.package]
version = "1.0.0"
edition = "2021"

[workspace]
members = [
  "crates/package"
]
resolver = "2"

[workspace.dependencies]
itertools = "^0.13.0"

[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
"#,
        )?;

        create_dir_all(workspace_dir.join("crates").join("package"))?;
        let package_manifest = workspace_dir
            .join("crates")
            .join("package")
            .join("Cargo.toml");
        File::create(&package_manifest)?.write_all(
            br#"
[package]
name = "package"
version.workspace = true
edition.workspace = true

[dependencies]
itertools = { workspace = true }

[lints]
workspace = true
"#,
        )?;

        Ok(TestWorkspace { package_manifest })
    }
}
