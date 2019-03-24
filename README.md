# cargo2nix

Generates nix build files from rust projects via `cargo metadata`.

Simple example:

```bash
# From the project directory.
cargo2nix generate >default.nix
```

More elaborate example that uses '<nixos-unstable>' as the default `nixpkgs` path:

```bash
cargo2nix generate \
    -n '<nixos-unstable>' \
    -f /some/project/dir/Cargo.toml \
    > /some/project/dir/cargo2nix.nix
```

## Known Restrictions

* Supports only default crate features. It should be easy to support a different feature set at build generation 
  time since we can simply pass this set to `cargo metadata`.
* Filters all dependencies for the hard-coded "Linux x86_64" target platform. Again, it should be quite easy to discover
  the current target via `rustc` at build generation time.
  
The actual nix code is generated via `templates/default.nix.tera` so potentially you can fix the nix code without
knowing much rust.

## Runtime Dependencies

cargo2nix use `cargo metadata` / `nix-prefetch-url` at runtime so they need to be in the PATH. 

cargo2nix depends on fixes in

* [cargo_metadata](https://github.com/oli-obk/cargo_metadata)
* [nixpkgs](https://github.com/NixOS/nixpkgs): [#56808](https://github.com/NixOS/nixpkgs/issues/56808)

We try to get these upstream.

## Related Projects

* [carnix](https://nest.pijul.com/pmeunier/carnix:master) is already widely used in nixos itself, yet it failed to
  generate correct builds for my rust projects.
* [cargo-raze](https://github.com/google/cargo-raze) generates `BUILD` files for bazel.

## Contributions

Contributions in the form of documentation and bug fixes are highly welcome. Please start a discussion with me before
working on larger features.

By submitting a pull request, you agree to license your changes via all the current licenses of the project.
