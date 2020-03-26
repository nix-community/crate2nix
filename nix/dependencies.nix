{ pkgs ? import ./nixpkgs.nix {}
, sources ? import ./sources.nix
  # Use last pinned crate2nix packages to build the test runner
  # so that it works even if we have broken stuff!
, tools ? pkgs.callPackage "${sources.crate2nix}/tools.nix" {}
}:

{
  dev = {

    inherit (pkgs)
      cargo clippy rustc rustfmt
      binutils
      nixpkgs-fmt jq
      nix
      git
      utillinux
      cacert
      ;

    nixTest = let
      nixTestRunnerCargoNix = tools.appliedCargoNix {
        name = "nix-test-runner";
        src = sources."nix-test-runner";
      };
    in
      nixTestRunnerCargoNix.rootCrate.build;
  };
}
