---
title: Generating Cargo.nix via IFD
---

* ✅ No need to install `crate2nix`.
* ✅ Auto-generates nix from your `Cargo.lock` file.
* ⚠️ Uses the [import from derivation](https://nixos.org/manual/nix/stable/language/import-from-derivation)
  feature from Nix which may lead to disabling build parallelism.

The `tools.nix` file contain the necessary code to generate the `Cargo.nix` file
during evaluation time, which guarantee to always have `Cargo.nix` file up-to-date
in regard to the `Cargo.lock`. The generated file is imported automatically in Nix
code via the [import from derivation feature](https://nixos.org/manual/nix/stable/language/import-from-derivation),
and can then be used like a normal `Cargo.nix` file.

Internally, this work by reading the `Cargo.lock` file with Nix, using the locked
version and hash present in it to fetch them without introducing impurities.
The fetched dependencies are then used to generate a vendored folder, and the
appropriate configuration is generated so that the dependencies are fetched from here.
`crate2nix` is then called in a derivation that will generate the `Cargo.nix` file
offline, which can later be imported.

There are two commands in the `tools.nix` file:

* `generatedCargoNix` will generate a folder containing a `default.nix`, to be used
  as a `Cargo.nix` file. The argument it takes are:
  * `name`: required. The name of the project (need to be a valid nix name
    identifier, so no space are allowed, but dash are.)
  * `src`: required. The folder that contain the root of the Rust project.
  * `cargoToml`: optional. The name and path relative to the `src` root of the
    `Cargo.toml` to use. Default to `Cargo.toml`.
  * `additionalCargoNixArgs`: optional, additional argument for `crate2nix`, in a list
* `appliedCargoNix` take the same argument that `generatedCargoNix`, but also call the
  generated file with the `pkgs` provided when calling `tools.nix`

for example:

```nix
let
  crate2nix-tools = pkgs.callPackage "${crate2nix}/tools.nix" {};
  generated = crate2nix-tools.generatedCargoNix {
    name = "test-rust-project";
    src = ./.;
  };
  called = pkgs.callPackage "${generated}/default.nix" {};
in
  called.rootCrate.build
```
