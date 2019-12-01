{ pkgs? import ../../nixpkgs.nix { config = {}; }
, generatedBuild ? ./Cargo.nix { } }:

let instantiatedBuild = pkgs.callPackage generatedBuild {};
in instantiatedBuild.rootCrate.build.override { runTests = true; }
