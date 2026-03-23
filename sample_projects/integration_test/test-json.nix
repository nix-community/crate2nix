# PR #453: test that JSON-mode buildTests wires dev-dependencies.
{ pkgs ? import ../../nix/nixpkgs.nix { config = { }; }
, generatedCargoNix ? null
}:
let
  cargoNix = import ../../lib/build-from-json.nix {
    inherit pkgs;
    src = ./.;
    resolvedJson = ./Cargo.json;
  };
in
cargoNix.rootCrate.buildTests
