{ pkgs ? import ../../nixpkgs.nix { config = {}; }, generatedCargoNix }:
let
  basePackage = pkgs.callPackage generatedCargoNix {
    rootFeatures = [ "default" "do_not_activate" ];
  };
  build = basePackage.rootCrate.build;
in
build
