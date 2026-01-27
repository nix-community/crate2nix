---
title: "Using crate overrides"
---

You can patch the individual crate derivations with `crateOverrides`, for
example to add native library dependencies such as openssl!

NixOS comes with
[`defaultCrateOverrides`](https://github.com/NixOS/nixpkgs/blob/master/pkgs/build-support/rust/default-crate-overrides.nix)
which specifies mostly some additional native `buildInputs` for various popular
crates. If you are using a rust crate with native dependencies which is not yet
covered, you can add additional `buildInputs` with the `crateOverride` parameter
(similar to `features`):

```nix
let
  customBuildRustCrateForPkgs = pkgs: pkgs.buildRustCrate.override {
    defaultCrateOverrides = pkgs.defaultCrateOverrides // {
      funky-things = attrs: {
        buildInputs = [ pkgs.openssl ];
      };
    };
  };
  generatedBuild = callPackage ./crate2nix/Cargo.nix {
    buildRustCrateForPkgs = customBuildRustCrateForPkgs;
  };
in generatedBuild.rootCrate.build
```

Or obviously you can use the power of nix to add a dependency conditionally:

```nix
let
  customBuildRustCrateForPkgs = pkgs: pkgs.buildRustCrate.override {
    defaultCrateOverrides = pkgs.defaultCrateOverrides // {
      cssparser-macros = attrs: {
        buildInputs =
          lib.optionals
            pkgs.stdenv.isDarwin
            [ pkgs.darwin.apple_sdk.frameworks.Security ];
      };
    };
  };
  generatedBuild = callPackage ./crate2nix/Cargo.nix {
    buildRustCrateForPkgs = customBuildRustCrateForPkgs;
  };
in generatedBuild.rootCrate.build
```

`crateOverrides` are not restricted to buildInputs however. You should also be
able to add patches and the like! (I didn't try that, though.)

`crateOverrides` are a feature of the underlying [`buildRustCrate` support in
NixOS](https://nixos.org/manual/nixpkgs/stable/#compiling-rust-crates-using-nix-instead-of-cargo)
that crate2nix uses.
