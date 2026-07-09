{ generatedCargoNix
, pkgs ? import ../../nix/nixpkgs.nix { crossSystem.config = "x86_64-unknown-linux-gnu"; }
}:
(pkgs.callPackage generatedCargoNix { }).workspaceMembers.a.build
