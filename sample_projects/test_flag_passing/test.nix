{ pkgs ? import ../../nix/nixpkgs.nix { config = { }; }
, generatedCargoNix ? ./Cargo.nix { }
, tools ? pkgs.callPackage ../../tools.nix { }
}:
let
  instantiatedBuild = pkgs.callPackage generatedCargoNix { };

  crate = instantiatedBuild.rootCrate.build;
in
tools.crateWithTest {
  inherit crate;
  testCrateFlags = [
    "--skip"
    "this_must_be_skipped"
  ];
}
