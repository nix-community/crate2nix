{ generatedCargoNix
, pkgs ? import ../../nix/nixpkgs.nix { crossSystem.config = "wasm32-unknown-none"; }
}:
(pkgs.callPackage generatedCargoNix { }).rootCrate.build
