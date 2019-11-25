{ pkgs? import ../../nixpkgs.nix { config = {}; } }:

let basePackage = pkgs.callPackage ./Cargo.nix {};
    submodulePackage = basePackage.rootCrate.build.override {
      crateOverrides = {
        librocksdb-sys = attrs: {
          src = (builtins.trace attrs.src attrs.src) + "/librocksdb-sys";
        };
      };
    }; in submodulePackage
