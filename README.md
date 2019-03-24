# cargo2nix

Generates nix build files from rust projects via `cargo metadata`.

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

## Known Restrictions

* Supports only default crate features. It should be easy to support a different feature set at build generation 
  time since we can simply pass this set to `cargo metadata`.
* Filters all dependencies for the hard-coded "Linux x86_64" target platform. Again, it should be quite easy to discover
  the current target via `rustc` at build generation time.
  
The actual nix code is generated via `templates/build.nix.tera` so potentially you can fix the nix code without
knowing much rust.

## Runtime Dependencies

cargo2nix use `cargo metadata` / `nix-prefetch-url` at runtime so they need to be in the PATH. The default.nix
adds the built-time nix/cargo binaries as fallback to the path.

Currently, cargo2nix is only tested with nixos-unstable (the future 19.09) since it depends on some new features
and bugfixes.

## Related Projects

* [carnix](https://nest.pijul.com/pmeunier/carnix:master) is already widely used in nixos itself, yet it failed to
  generate correct builds for my rust projects.
* [cargo-raze](https://github.com/google/cargo-raze) generates `BUILD` files for bazel.

## Contributions

Contributions in the form of documentation and bug fixes are highly welcome. Please start a discussion with me before
working on larger features.

By submitting a pull request, you agree to license your changes via all the current licenses of the project.
