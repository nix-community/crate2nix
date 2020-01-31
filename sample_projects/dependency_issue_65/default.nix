{ pkgs? import ../../nixpkgs.nix { config = {}; }
, generatedBuild ? ./Cargo.nix { } }:

let cargo_nix = pkgs.callPackage generatedBuild {};
    bin = cargo_nix.rootCrate.build.override { features = ["sqlite"]; };
in bin