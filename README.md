# cargo2nix

Generates nix build files with `buildRustCrate` derivations from rust projects via `cargo metadata`.

[![Build Status](https://travis-ci.org/kolloch/cargo2nix.svg?branch=master)](https://travis-ci.org/kolloch/cargo2nix)

Simple example:

```bash
# From the project directory.
cargo2nix generate
```

More elaborate example that uses '<nixos-unstable>' as the default `nixpkgs` path:

```bash
cargo2nix generate \
    -n '<nixos-unstable>' \
    -f /some/project/dir/Cargo.toml \
    -o /some/project/dir/cargo2nix.nix
```

## Installation

For now, clone the repository and then

```bash
cd cargo2nix
nix-shell
# you are in a shell with cargo2nix
```

This assumes that nixos-unstable points to, well, nixos-unstable.

If that doesn't work for you, you can either add it to your nix channels:

```bash
nix-channel --add https://nixos.org/channels/nixos-unstable nixos-unstable
```

Or you override the pkgs argument, e.g.:

```bash
nix-shell --arg pkgs 'import <nixos> {config = {}; }'
```

## Known Restrictions

* Only *default crate features* are supported. It should be easy to support a different feature set at build generation 
  time since we can simply pass this set to `cargo metadata`. Feature selection during build time is out of scope for 
  now.
* Filters all dependencies for the *hard-coded "Linux x86_64" target platform*. Again, it should be quite easy to 
  discover the current target via `rustc` at build generation time.
* Only *local sources* and *crates io* supported. Again, just requires some work to resolve.
  
The actual nix code is generated via `templates/build.nix.tera` so potentially you can fix the nix code without
knowing much rust.

## Runtime Dependencies

cargo2nix use `cargo metadata` / `nix-prefetch-url` at runtime so they need to be in the PATH. The default.nix
adds the built-time nix/cargo binaries as fallback to the path.

Currently, cargo2nix is only tested with nixos-unstable (the future 19.03) since it depends on some new features
and bug fixes.

## Related Projects

* [carnix](https://nest.pijul.com/pmeunier/carnix:master) is already widely used in nixos itself, yet it failed to
  generate correct builds for my rust projects. That said, big kudos for all the work on buildRustCrate!
* [cargo-raze](https://github.com/google/cargo-raze) generates `BUILD` files for bazel.

## Project Overview / Terminology

If you want to hack on this, it is useful to know that build file generation is broken up into multiple phases:

* Calling `cargo metadata` via the `cargo_metadata` crate.
* Indexing the metadata by package ID to enable easy joining of "Node" and "Package" information, resulting in 
  `metadata::IndexedMetadata`.
* Using the indexed metadata to actually resolve the dependencies and join all needed build information into 
  `resolve::CrateDerivation`.
* Pre-fetching crates.io packages to determine their sha256, see `prefetch` module.
* Rendering the data via the `build.nix.tera` template, see `render` module.

## Contributions

Contributions in the form of documentation and bug fixes are highly welcome. Please start a discussion with me before
working on larger features.

Feature ideas are also welcome -- just know that this is a pure hobby side project and I will not allocate a lot of
bandwidth to this. Therefore, important bug fixes are always prioritised.

By submitting a pull request, you agree to license your changes via all the current licenses of the project.
