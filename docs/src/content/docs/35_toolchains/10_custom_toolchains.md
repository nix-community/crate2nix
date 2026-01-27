---
title: Custom Toolchains
---

One way to use a custom rust toolchain with `crate2nix` is to
import nixpkgs with an overlay for `rustc`.

Here is an example flake using [fenix]:

```nix

{
  description = "containix";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/24.05";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    crate2nix.url = "github:nix-community/crate2nix";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      fenix,
      crate2nix,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        toolchain = fenix.packages.${system}.stable.defaultToolchain;

        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            (final: prev: {
              rustc = toolchain;
              cargo = toolchain;
            })
          ];
        };

        crate2nix' = pkgs.callPackage (import "${crate2nix}/tools.nix") {};
        cargoNix = crate2nix'.appliedCargoNix {
          name = "my-crate";
          src = ./.;
        };
      in
      {
        packages = {
          default = cargoNix.rootCrate.build;
        };
      }
    );
}

```

[fenix]: https://github.com/nix-community/fenix
