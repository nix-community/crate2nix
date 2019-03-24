#
# Crates a shell with cargo2nix installed.
#

{ pkgs? import <nixos-unstable> { config = {}; }}:

let cargo2nix = pkgs.callPackage ./default.nix {};
in pkgs.stdenv.mkDerivation {
   name = "shell-with-cargo2nix";
   src = ./.;

   buildInputs = [ cargo2nix ];

   shellHook = ''
    source ${cargo2nix}/share/bash-completion/completions/cargo2nix.bash
    cargo2nix help
   '';
 }