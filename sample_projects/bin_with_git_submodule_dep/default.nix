{ pkgs? import <nixpkgs> { config = {}; } }:

let basePackage = pkgs.callPackage ./Cargo.nix {};
    submodulePackage = basePackage.rootCrate.build.override {
      crateOverrides = {
        librocksdb-sys = attrs: {
          src = attrs.src + "/librocksdb-sys";
        };
      };
    }; in submodulePackage
