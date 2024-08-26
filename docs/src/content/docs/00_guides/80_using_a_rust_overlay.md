---
title: Using a Rust overlay
---

It's possible to use a Rust overlay such as `rust-overlay` with `crate2nix`.
This can be used for pinning the Rust toolchain version, or to use a newer toolchain version than is available in Nixpkgs.

In a flake with flake-parts:

```nix
# flake.nix
{
  # ...

  inputs = {
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    crate2nix.url = "github:nix-community/crate2nix";
    # ...
  };
  
  outputs =
    inputs @ { self
    , nixpkgs
    , flake-parts
    , rust-overlay
    , crate2nix
    , ...
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-linux"
        "aarch64-darwin"
      ];

      perSystem = { system, lib, inputs', ... }:
        let
          pkgs = import nixpkgs {
            inherit system;
            
            overlays = [ rust-overlay.overlays.default ];
          };
          
          buildRustCrateForPkgs =
            crate:
            pkgs.buildRustCrate.override {
              rustc = pkgs.rust-bin.stable.latest.default;
              cargo = pkgs.rust-bin.stable.latest.default;
            };
            
          generatedCargoNix = inputs.crate2nix.tools.${system}.appliedCargoNix {
            name = "rustnix";
            src = ./.;
          };
          
          cargoNix = import generatedCargoNix {
            inherit pkgs buildRustCrateForPkgs;
          };
        in
          rec {
            checks = {
              rustnix = cargoNix.rootCrate.build.override {
                runTests = true;
              };
            };
  
            packages = {
              rustnix = cargoNix.rootCrate.build;
              default = packages.rustnix;
            };
          };
    };
}
```
