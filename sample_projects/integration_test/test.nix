{ pkgs ? import ../../nix/nixpkgs.nix { config = {}; }
, generatedCargoNix ? ./Cargo.nix {}
}:
let
  instantiatedBuild = pkgs.callPackage generatedCargoNix {};
in
instantiatedBuild.rootCrate.build.override {
  runTests = true;
}
