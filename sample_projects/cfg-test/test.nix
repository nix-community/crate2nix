{ pkgs? import ../../nixpkgs.nix { config = {}; } }:
let
  basePackage = pkgs.callPackage ./Cargo.nix { };
  submodulePackage = basePackage.rootCrate.build.override { runTests = true; };
in submodulePackage
