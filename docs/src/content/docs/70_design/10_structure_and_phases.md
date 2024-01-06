---
title: Project Overview & Terminology
---

If you want to hack on this, it is useful to know that build file generation is broken up into multiple phases:

1. **cargo metadata**: Calling `cargo metadata` via the `cargo_metadata` crate.
2. **indexing metadata**: Indexing the metadata by package ID to enable easy joining of "Node" and "Package"
  information, resulting in `metadata::IndexedMetadata`.
3. **resolving**: Using the indexed metadata to actually resolve the dependencies and join all needed build information
  into `resolve::CrateDerivation`.
4. **pre-fetching**: Pre-fetching crates.io packages to determine their sha256, see `prefetch` module.
5. **rendering**: Rendering the data via the `build.nix.tera` template, see `render` module.
