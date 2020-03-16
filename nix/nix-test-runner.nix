{ pkgs ? import ./nixpkgs.nix {}
, sources ? import ./sources.nix
  # Use last pinned crate2nix packages to build the test runner
  # so that it works even if we have broken stuff!
, tools ? pkgs.callPackage "${sources.crate2nix}/tools.nix" {}
}:
let
  nixTestRunner = tools.appliedCargoNix {
    name = "nix-test-runner";
    src = sources."nix-test-runner";
  };
in
nixTestRunner.rootCrate.build
