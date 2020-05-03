{ sources ? import ./sources.nix
, pkgs ? import sources.nixpkgs { }
  # Use last pinned crate2nix packages to build the test runner
  # so that it works even if we have broken stuff!
, tools ? pkgs.callPackage "${sources.crate2nix}/tools.nix" { }
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

    niv = (pkgs.callPackage sources.niv {}).niv;

    nixTest =
      let
        cargoNix = tools.appliedCargoNix rec {
          name = "nix-test-runner";
          src = sources."${name}";
        };
      in
      cargoNix.rootCrate.build;

    cargoRelease =
      let
        cargoNix = tools.appliedCargoNix rec {
          name = "cargo-release";
          src = sources."${name}";
        };
      in
      cargoNix.rootCrate.build;
  };
}
