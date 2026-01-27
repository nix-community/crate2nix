{ generatedCargoNix
, pkgs ? import ../../nix/nixpkgs.nix { config = { }; }
}:
let
  pkgs0 = pkgs;
in
let
  pkgs = import pkgs0.path {
    crossSystem = {
      config = "x86_64-unknown-linux-gnu";
    };
  };
in
(pkgs.callPackage generatedCargoNix { }).workspaceMembers.a.build
