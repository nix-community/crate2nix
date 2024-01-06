---
title: Inspiration & History
description: Where did crate2nix come from?
---

[carnix](https://nest.pijul.com/pmeunier/carnix:master) was already widely used
in NixOS itself and also built projects at crate granularity!

Unfortunately, it failed to generate correct builds for
[@kolloch's](https://github.com/kolloch) rust projects. After some attempts to
fix `carnix` itself, he gave up and started `crate2nix`. That said, big kudos for all the
work on `buildRustCrate` and showing the way!

The [NixOS Wiki Rust Page](https://nixos.wiki/wiki/Rust#Packaging_Rust_projects_with_nix)
contains a nice overview over currently available approaches for building
rust projects with nix.

## Other Related Projects

* [naersk](https://github.com/nmattia/naersk/) uses cargo to drive the
  entire build. It builds all dependencies in one derivation and the crate itself in another.
  Since it relies on hashes from the Cargo.lock
  file, I don't know how it handles git dependencies with sub modules.
* [tenx-tech/cargo2nix](https://github.com/tenx-tech/cargo2nix): I
  haven't used it so take it with a grain of salt but I think
  * it uses its own build logic instead of `buildRustCrate` but
      still builds each crate in its own derivation.
  * it has some support for cross building (which is quite weak in
      crate2nix).
* [cargo-raze](https://github.com/google/cargo-raze) generates `BUILD`
  files for bazel.
