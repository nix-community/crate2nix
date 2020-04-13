{ pkgs ? import ./nix/nixpkgs.nix }:
let
  cargoNix = pkgs.callPackage ./Cargo.nix {};
in
cargoNix.rootCrate.build
