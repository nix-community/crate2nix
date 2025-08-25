{ generatedCargoNix
, pkgs ? import ../../nix/nixpkgs.nix { config = { }; }
}:
let
  pkgs0 = pkgs;
in
let
  pkgs = import pkgs0.path {
    crossSystem = {
      config = "wasm32-wasi";
      rust = {
        rustcTarget = "wasm32-wasip1";
        rustcTargetSpec = "wasm32-wasip1";
        # https://github.com/NixOS/nixpkgs/issues/436832
        platform = {
          arch = "wasm32";
          os = "wasi";
          env = "p1";
        };
      };
    };
  };
in
(pkgs.callPackage generatedCargoNix { }).workspaceMembers.a.build
