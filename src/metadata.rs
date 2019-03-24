//! Indexing cargo metadata.

use std::collections::BTreeMap;

use cargo_metadata::Metadata;
use cargo_metadata::Node;
use cargo_metadata::Package;
use cargo_metadata::PackageId;
use failure::format_err;
use failure::Error;
use serde_derive::Deserialize;
use serde_derive::Serialize;

/// The metadata with maps indexed by {{PackageId}} instead of flat lists.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IndexedMetadata {
    pub root: Option<PackageId>,
    pub pkgs_by_id: BTreeMap<PackageId, Package>,
    pub nodes_by_id: BTreeMap<PackageId, Node>,
}

impl IndexedMetadata {
    pub fn new_from(metadata: Metadata) -> Result<IndexedMetadata, Error> {
        let resolve = metadata
            .resolve
            .ok_or_else(|| format_err!("no root in metadata"))?;

        let pkgs_by_id: BTreeMap<PackageId, Package> = metadata
            .packages
            .iter()
            .map(|pkg| (pkg.id.clone(), pkg.clone()))
            .collect();

        if pkgs_by_id.len() != metadata.packages.len() {
            let duplicate_ids =
                crate::util::find_duplicates(metadata.packages.iter().map(|p| &p.id));
            return Err(format_err!(
                "detected duplicate package IDs in metadata.packages: {:?}",
                duplicate_ids
            ));
        }

        let nodes_by_id: BTreeMap<PackageId, Node> = resolve
            .nodes
            .iter()
            .map(|node| (node.id.clone(), node.clone()))
            .collect();

        if nodes_by_id.len() != resolve.nodes.len() {
            let duplicate_ids = crate::util::find_duplicates(resolve.nodes.iter().map(|n| &n.id));
            return Err(format_err!(
                "detected duplicate package IDs in nodes: {:?}",
                duplicate_ids
            ));
        }

        Ok(IndexedMetadata {
            root: resolve.root,
            pkgs_by_id,
            nodes_by_id,
        })
    }
}
