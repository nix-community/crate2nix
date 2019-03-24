//! Utilities for calling `nix-prefetch` on packages.

use std::io::Write;
use std::process::Command;

use crate::GenerateConfig;
use cargo_metadata::{Package, PackageId, Source};
use failure::bail;
use failure::format_err;
use failure::Error;
use std::collections::BTreeMap;

/// Uses `nix-prefetch` to get the hashes of the sources for the given packages if they come from crates.io.
///
/// Uses and updates the existing hashes in the `config.crate_hash_json` file.
pub fn prefetch_packages<'a>(
    config: &GenerateConfig,
    packages: impl Iterator<Item = &'a Package>,
) -> Result<BTreeMap<PackageId, String>, Error> {
    let hashes_string: String =
        std::fs::read_to_string(&config.crate_hashes_json).unwrap_or_else(|_| "{}".to_string());

    let old_hashes: BTreeMap<PackageId, String> = serde_json::from_str(&hashes_string)?;
    // Only copy used hashes over to the new map.
    let mut hashes: BTreeMap<PackageId, String> = BTreeMap::new();

    // Skip none-registry packages.
    let packages_from_crates_io: Vec<&'a Package> = packages
        .filter(|p| p.source.as_ref().map(Source::is_crates_io).unwrap_or(false))
        .collect();
    let without_hash_num = packages_from_crates_io
        .iter()
        .filter(|p| !old_hashes.contains_key(&p.id))
        .count();
    let mut without_hash_idx = 0;
    for package in packages_from_crates_io {
        let existing_hash = old_hashes.get(&package.id);
        let hash = if let Some(hash) = existing_hash {
            hash.trim().to_string()
        } else {
            without_hash_idx += 1;
            crate::prefetch::nix_prefetch(package, without_hash_idx, without_hash_num)?
        };

        hashes.insert(package.id.clone(), hash);
    }

    if hashes != old_hashes {
        std::fs::write(
            &config.crate_hashes_json,
            serde_json::to_vec_pretty(&hashes)?,
        )?;
        eprintln!(
            "Wrote hashes to {}.",
            config.crate_hashes_json.to_string_lossy()
        );
    }

    Ok(hashes)
}

/// Invoke `nix-prefetch` for the given `package` and return the hash.
pub fn nix_prefetch(package: &Package, idx: usize, num_packages: usize) -> Result<String, Error> {
    let url = format!(
        "https://crates.io/api/v1/crates/{}/{}/download",
        package.name, package.version
    );

    eprintln!("Prefetching {:>4}/{}: {}", idx, num_packages, url);
    let cmd = "nix-prefetch-url";
    let args = [
        &url,
        "--unpack",
        "--name",
        &format!("{}-{}", package.name, package.version),
    ];
    let output = Command::new(cmd)
        .args(&args)
        .output()
        .map_err(|e| format_err!("While spawning '{} {}': {}", cmd, args.join(" "), e))?;

    if !output.status.success() {
        std::io::stdout().write_all(&output.stdout)?;
        std::io::stderr().write_all(&output.stderr)?;
        bail!(
            "{}\n=> exited with: {}",
            "nix-prefetch-url",
            output.status.code().unwrap_or(-1)
        );
    }

    Ok(String::from_utf8(output.stdout)
        .map(|s| s.trim().to_string())
        .map_err(|_e| format_err!("output of '{} {}' is not UTF8!", cmd, args.join(" ")))?)
}
