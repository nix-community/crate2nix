# Build crate2nix from the pre-resolved JSON output, dogfooding the JSON path.
{ pkgs ? (
    import (builtins.fetchTree (import ../nix/flakeInput.nix "nixpkgs")) { }
  )
, stdenv ? pkgs.stdenv
, lib ? pkgs.lib
, symlinkJoin ? pkgs.symlinkJoin
, makeWrapper ? pkgs.makeWrapper
, nix ? pkgs.nix
, cargo ? pkgs.cargo
, libsecret ? pkgs.libsecret
, nix-prefetch-git ? pkgs.nix-prefetch-git
, release ? true
}:
let
  cargoNix = import ../lib/build-from-json.nix {
    inherit pkgs lib stdenv;
    src = ./.;
    resolvedJson = ./Cargo.json;
  };
in
import ./mk-crate2nix.nix {
  inherit pkgs stdenv lib symlinkJoin makeWrapper nix cargo libsecret
    nix-prefetch-git release;
  rootCrate = cargoNix.rootCrate.build;
}
