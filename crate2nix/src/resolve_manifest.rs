//! Expand a Cargo.toml manifest to include any data inherited from its workspace.

use std::path::Path;

use cargo_toml::Manifest;

/// Loads a Cargo.toml manifest (which may be a workspace member manifest) from the given path.
/// Automatically locates the corresponding manifest if there is one to fill in inherited values.
///
/// For example `version.workspace = true` becomes `version = "1.2.3"`
pub fn resolve_manifest(cargo_toml: &Path) -> Result<Manifest, cargo_toml::Error> {
    Manifest::from_path(cargo_toml)
}
