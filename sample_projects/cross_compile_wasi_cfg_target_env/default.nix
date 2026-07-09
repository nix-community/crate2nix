{ generatedCargoNix
, pkgs ? import ../../nix/nixpkgs.nix { crossSystem.config = "wasm32-wasi"; }
}:
(pkgs.callPackage generatedCargoNix { }).workspaceMembers.a.build
