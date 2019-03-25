# crate2nix

Generates [nix](https://nixos.org/nix/) build files for building [rust](https://www.rust-lang.org/) binaries from 
[cargo](https://crates.io/) projects.

**Same dependency tree as cargo**: It uses [cargo_metadata](https://github.com/oli-obk/cargo_metadata) to obtain the dependency tree from cargo. Therefore,
it will use the exact same library versions as cargo and respect any locked down version in `Cargo.lock`.

**Smart caching**: It uses smart crate by crate caching so that nix rebuilds exactly the crates that need to be rebuilt. You can use all
things that make the nix ecosystem great, e.g. deploy your binary as a service to the the cloud with 
[NixOps](https://nixos.org/nixops/).

**Out of the box support for libraries with non-rust dependencies**: It builds on top of the `buildRustCrate` 
function from [NixOS](https://nixos.org/) so that native dependencies of
many rust libraries are already correctly fetched when needed. If your library with native dependencies is not yet 
supported, you can create an overlay to add the needed configuration to the `defaultCrateOverrides`.

**Easy to understand nix template**: The actual nix code is generated via `templates/build.nix.tera` so you can 
fix/improve the nix code without knowing rust if all the data is already there.

[![Build Status](https://travis-ci.org/kolloch/crate2nix.svg?branch=master)](https://travis-ci.org/kolloch/crate2nix)

Simple example:

```bash
# From the project directory.
crate2nix generate
```

More elaborate example that uses `<nixos-unstable>` as the default `nixpkgs` path and specifies both the path
to the `Cargo.toml` file (`-f`) and the output (`-o`) file explicitly.

```bash
crate2nix generate \
    -n '<nixos-unstable>' \
    -f /some/project/dir/Cargo.toml \
    -o /some/project/dir/crate2nix.nix
```

Use `crate2nix help` to show all commands and options.

## Installation

For now, clone the repository and then

```bash
# Install nix if necessary: https://nixos.org/nix/
cd crate2nix
nix-shell
# you are in a shell with crate2nix
```

This assumes that the `<nixos-unstable>` path points to, well, nixos-unstable.

If that doesn't work for you, you can either 

1. either add it to your nix channels:

```bash
nix-channel --add https://nixos.org/channels/nixos-unstable nixos-unstable
```

2. or you override the `pkgs` argument, e.g.:

```bash
nix-shell --arg pkgs 'import <nixos> {config = {}; }'
```

## Known Restrictions

* Only *default crate features* are supported. It should be easy to support a different feature set at build generation 
  time since we can simply pass this set to `cargo metadata`. Feature selection during build time is out of scope for 
  now.
* Filters all dependencies for the *hard-coded "Linux x86_64" target platform*. Again, it should be quite easy to 
  support more platforms. To do so completely and at build time (vs build generation time) might be more involved.
* Only *local sources* and *crates io* supported. Again, just requires some work to resolve.
* Since cargo exposes local paths in package IDs, the generated build file also contain them as part of an "opaque"
  ID. They are not interpreted as paths but maybe you do not want to expose local paths in there...

## Runtime Dependencies

crate2nix use `cargo metadata` / `nix-prefetch-url` at runtime so they need to be in the PATH. The default.nix
adds the built-time nix/cargo binaries as fallback to the path.

Currently, crate2nix is only tested with nixos-unstable (the future 19.03) since it depends on some new features
and bug fixes.

## Related Projects

* [carnix](https://nest.pijul.com/pmeunier/carnix:master) is already widely used in NixOS itself, yet it failed to
  generate correct builds for my rust projects. After some attempts to fix that, I gave up. That said, big kudos for 
  all the work on buildRustCrate and showing the way!
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
