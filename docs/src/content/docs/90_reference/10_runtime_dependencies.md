---
title: Runtime Dependencies
---

crate2nix uses `cargo metadata` at runtime.

Depending on the situation it also calls out to

* `nix-prefetch-url` (e.g. for git dependencies),
* `nix` (e.g. for out of tree functionality).

The default package appends the nixpkgs default versions of all runtime
dependencies to the path, so that they should never be missing.
