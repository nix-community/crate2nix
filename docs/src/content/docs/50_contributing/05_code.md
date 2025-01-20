---
title: Nix/Rust/Tera Code
description: Some pointers for contributing code successfully.
---

Read [Project Overview & Terminology](/crate2nix/70_design/10_structure_and_phases/)
for a first overview.

We really appreciate tests for all new features. Please run `./run_tests.sh`
before submitting a pull request and/or react to test failures in the pull
request.

## Running tests

`run_tests.sh` will regenerate build files AND run cargo test for you. It will
call out to nix to build the sample projects -- so a considerable number of
dependencies will be fetched and built. Consecutive runs are much faster.

### Running an individual integration test

:::tip[Sample projects]
Many of these just build a sample project in the
[sample_projects folder](https://github.com/nix-community/crate2nix/tree/master/sample_projects).
:::

To run just an individual test (in this example "bin_with_lib_dep"):

```shell
nix-build \
  -o ./crate2nix/target/nix-results ./tests.nix -A bin_with_lib_dep
```

If you want to run a test that is currently marked as skip, you can do so
with:

```shell
nix-build \
    -o ./crate2nix/target/nix-results ./tests.nix -A empty_cross.forceSkipped
```

### Regenerating Cargo.nix files for tests

If you change `crate2nix` such that it will produce a different output, you may
need to regenerate some of the Cargo.toml files. Not all `Cargo.toml` files can
be generated during test time because crate2nix needs to vendor the dependencies
for this to work and support for git submodules is not working yet, see
[#101](https://github.com/nix-community/crate2nix/issues/101).

`regenerate_cargo_nix.sh` should do what you want and is run as part of
`run_tests.sh` (see below).

### Updating `crate-hashes.json`

`regenerate_cargo_nix.sh` does not automatically refetch any references
if you update a `Cargo.lock` file of a sample_project. You need
to call `crate2nix generate` manually. `crate-hashes.json` will be updated,
the generated Cargo.nix will be overwritten by `./regenerate_cargo_nix.sh`.

## Hacking on `buildRustCrate` in nixpkgs

Since `crate2nix` heavily depends on `buildRustCrate` in `nixpkgs`, it makes
sense to hack on them together.

To be able to provide pull requests, you probably want to fork
[nixpkgs](https://github.com/NixOS/nixpkgs) first. Once you have done that,
clone that fork into some local directory (separate from crate2nix).

### Overriding nixpkgs for everything

Now you can run the integration tests of `crate2nix` against that version of
nixpkgs. Let's assume you are in the `crate2nix` project directory and you
cloned `nixpkgs` to a sibling directory:

```shell
nix-build --arg nixpkgs ../nixpkgs -o ./crate2nix/target/nix-results ./tests.nix
```

Or run just an individual test (in this example "bin_with_lib_dep"):

```shell
nix-build --arg nixpkgs ../nixpkgs \
  -o ./crate2nix/target/nix-results ./tests.nix -A bin_with_lib_dep
```

(The "-o" argument is just to avoid a lot of top-level result directories.)

### Overriding nixpkgs for buildTests

The problems is that this method will rebuild everything with your nixpkgs,
including `crate2nix` itself. That can become severely annoying if you want
to iterate on one special test case.

If you are fixing an issue in `buildRustCrate` that you can reproduce with a
`buildTest` in `tests.nix`, then there is a much better way.

```shell
nix-build --arg buildTestNixpkgs ../nixpkgs \
  -o ./crate2nix/target/nix-results ./tests.nix -A bin_with_lib_dep
```

Notice `--arg buildTestNixpkgs` instead of `--arg nixpkgs`. That will not
rebuild `crate2nix` itself with your nixpkgs but it will use `buildRustCrate`
from your `nixpkgs` for all `buildTests`.
