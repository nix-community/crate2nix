let
  flakeInput = import ./flakeInput.nix;
  src = builtins.fetchTree (flakeInput "nix-test-runner");

  # Use last pinned crate2nix packages and corresponding nixpkgs to build the
  # test runner so that it works even if we have broken stuff!
  crate2nix_stable = builtins.fetchTree (flakeInput "crate2nix_stable");
  nixpkgs_stable = builtins.fetchTree (flakeInput "crate2nix_stable.nixpkgs");
in
{ system ? builtins.currentSystem
, pkgs ? import nixpkgs_stable { inherit system; }
, tools ? pkgs.callPackage "${crate2nix_stable}/tools.nix" { }
}:
let
  nixTestRunner = tools.appliedCargoNix {
    name = "nix-test-runner";
    inherit src;
  };
in
nixTestRunner.rootCrate.build
