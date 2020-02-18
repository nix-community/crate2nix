//! Indexing cargo metadata.

use std::collections::{BTreeMap, HashMap};

use cargo_metadata::Node;
use cargo_metadata::Package;
use cargo_metadata::PackageId;
use cargo_metadata::{Metadata, NodeDep};
use failure::format_err;
use failure::Error;
use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;

/// The metadata with maps indexed by {{PackageId}} instead of flat lists.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IndexedMetadata {
    pub root: Option<PackageId>,
    pub workspace_members: Vec<PackageId>,
    pub pkgs_by_id: BTreeMap<PackageId, Package>,
    pub nodes_by_id: BTreeMap<PackageId, Node>,
    pub id_shortener: PackageIdShortener,
}

impl IndexedMetadata {
    pub fn new_from(metadata: Metadata) -> Result<IndexedMetadata, Error> {
        let resolve = metadata
            .resolve
            .as_ref()
            .ok_or_else(|| format_err!("no root in metadata"))?;

        let id_shortener = PackageIdShortener::new(metadata.packages.iter());

        let pkgs_by_id: BTreeMap<PackageId, Package> = metadata
            .packages
            .iter()
            .map(|pkg| {
                (
                    id_shortener.shorten(&pkg.id),
                    id_shortener.shorten_in_package(pkg),
                )
            })
            .collect();

        if pkgs_by_id.len() != metadata.packages.len() {
            let duplicate_ids = crate::util::find_duplicates(
                metadata
                    .packages
                    .iter()
                    .map(|p| &id_shortener.shorten_ref(&p.id).repr),
            );
            return Err(format_err!(
                "detected duplicate package IDs in metadata.packages: {:?}",
                duplicate_ids
            ));
        }

        let nodes_by_id: BTreeMap<PackageId, Node> = resolve
            .nodes
            .iter()
            .map(|node| {
                (
                    id_shortener.shorten(&node.id),
                    id_shortener.shorten_in_node(&node),
                )
            })
            .collect();

        if nodes_by_id.len() != resolve.nodes.len() {
            let duplicate_ids = crate::util::find_duplicates(
                resolve
                    .nodes
                    .iter()
                    .map(|n| &id_shortener.shorten_ref(&n.id).repr),
            );
            return Err(format_err!(
                "detected duplicate package IDs in nodes: {:?}",
                duplicate_ids
            ));
        }

        Ok(IndexedMetadata {
            root: resolve.root.as_ref().map(|id| id_shortener.shorten(&id)),
            workspace_members: metadata
                .workspace_members
                .iter()
                .map(|id| id_shortener.shorten(&id))
                .collect(),
            pkgs_by_id,
            nodes_by_id,
            id_shortener,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PackageIdShortener {
    substitution: HashMap<PackageId, PackageId>,
    reverse: HashMap<PackageId, PackageId>,
}

impl PackageIdShortener {
    /// Returns a substitution map for shorter package IDs. It falls back to the next
    /// longer option if it is not unique. The options in order:
    ///
    /// * Just the crate name.
    /// * The crate name and the version.
    ///
    /// If the shortening substitution wasn't successful, the package_id is not contained in
    /// the returned map.
    #[allow(clippy::needless_lifetimes)]
    fn new<'a>(all_packages: impl Iterator<Item = &'a Package>) -> PackageIdShortener {
        let mut substitution = HashMap::new();
        let mut reverse = HashMap::new();

        for (_crate_name, group) in &all_packages
            .sorted_by_key(|p| &p.name)
            .group_by(|p| p.name.clone())
        {
            let packages: Vec<_> = group.collect();

            enum UniqueComponent {
                Name,
                NameVersion,
                PackageId,
            };
            let unique_component = if packages.len() == 1 {
                UniqueComponent::Name
            } else if packages.iter().map(|p| &p.version).unique().count() == packages.len() {
                UniqueComponent::NameVersion
            } else {
                UniqueComponent::PackageId
            };

            for p in &packages {
                let substitute = match unique_component {
                    UniqueComponent::Name => Some(p.name.clone()),
                    UniqueComponent::NameVersion => Some(format!("{} {}", p.name, p.version)),
                    UniqueComponent::PackageId => None,
                };
                if let Some(repr) = substitute {
                    substitution.insert(p.id.clone(), PackageId { repr: repr.clone() });
                    reverse.insert(PackageId { repr }, p.id.clone());
                }
            }
        }

        PackageIdShortener {
            substitution,
            reverse,
        }
    }

    pub fn lengthen_ref<'a>(&'a self, package_id: &'a PackageId) -> &'a PackageId {
        self.reverse.get(&package_id).unwrap_or_else(|| &package_id)
    }

    pub fn shorten_ref<'a>(&'a self, package_id: &'a PackageId) -> &'a PackageId {
        self.substitution
            .get(&package_id)
            .unwrap_or_else(|| &package_id)
    }

    pub fn shorten(&self, package_id: &PackageId) -> PackageId {
        self.substitution
            .get(&package_id)
            .cloned()
            .unwrap_or_else(|| package_id.clone())
    }

    pub fn shorten_owned(&self, package_id: PackageId) -> PackageId {
        self.substitution
            .get(&package_id)
            .cloned()
            .unwrap_or_else(|| package_id)
    }

    fn shorten_in_package(&self, package: &Package) -> Package {
        let mut p = package.clone();
        p.id = self.shorten(&package.id);
        p
    }

    fn shorten_in_node(&self, node: &Node) -> Node {
        let mut n = node.clone();
        n.id = self.shorten_owned(n.id);
        n.dependencies = n
            .dependencies
            .into_iter()
            .map(|id| self.shorten_owned(id))
            .collect();
        n.deps = n
            .deps
            .iter()
            .map(|dep| self.shorten_in_node_dep(dep))
            .collect();
        n
    }

    fn shorten_in_node_dep(&self, nod_dep: &NodeDep) -> NodeDep {
        let mut n = nod_dep.clone();
        n.pkg = self.shorten_owned(n.pkg);
        n
    }
}
