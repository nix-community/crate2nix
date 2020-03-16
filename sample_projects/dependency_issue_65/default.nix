{ pkgs ? import ../../nix/nixpkgs.nix { config = {}; }
, generatedCargoNix ? ./Cargo.nix {}
}:
let
  cargo_nix = pkgs.callPackage generatedCargoNix {};
  bin = cargo_nix.rootCrate.build.override { features = [ "sqlite" ]; };
in
bin
