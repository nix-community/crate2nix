{ pkgs? import ../../nixpkgs.nix { config = {}; } }:

let customBuildRustCrate = pkgs.buildRustCrate.override {
  defaultCrateOverrides = pkgs.defaultCrateOverrides // {
    librocksdb-sys = attrs: {
      src = attrs.src + "/librocksdb-sys";
    };
  };
};
basePackage = pkgs.callPackage ./Cargo.nix { buildRustCrate = customBuildRustCrate; };
submodulePackage = basePackage.rootCrate.build;
in submodulePackage
