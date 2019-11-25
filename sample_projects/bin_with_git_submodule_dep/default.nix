{ pkgs? import ../../nixpkgs.nix { config = {}; } }:

let buildRustCrate = pkgs.buildRustCrate.override {
  defaultCrateOverrides = pkgs.defaultCrateOverrides // {
    librocksdb-sys = attrs: {
      src = attrs.src + "/librocksdb-sys";
    };
  };
};
basePackage = pkgs.callPackage ./Cargo.nix {};
submodulePackage = basePackage.rootCrate.build;
in submodulePackage
