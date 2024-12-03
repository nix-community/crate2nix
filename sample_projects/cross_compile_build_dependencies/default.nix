{ nixpkgs ? import ../../nix/nixpkgs.nix
, lib ? import "${nixpkgs}/lib"
, pkgs ? import ../../nix/nixpkgs.nix { crossSystem = lib.systems.examples.wasm32-unknown-none; }
, generatedCargoNix
}:
(pkgs.callPackage generatedCargoNix { }).workspaceMembers.alice.build
