let
  lib = import ./lib.nix;
  src = builtins.fetchTree (lib.flakeInput "nix-test-runner");
in
{ system ? null # a system must be specified if using default value for pkgs

  # Use last pinned crate2nix packages and corresponding nixpkgs to build the
  # test runner so that it works even if we have broken stuff!
, pkgs ? import (builtins.fetchTree (lib.flakeNestedInput [ "crate2nix_stable" "nixpkgs" ])) { inherit system; }
, tools ? pkgs.callPackage "${builtins.fetchTree (lib.flakeInput "crate2nix_stable")}/tools.nix" { }
}:
let
  nixTestRunner = tools.appliedCargoNix {
    name = "nix-test-runner";
    inherit src;
  };
in
nixTestRunner.rootCrate.build
