#
# Crates a shell with crate2nix installed.
#

let nixpkgs = builtins.fetchGit { url = https://github.com/NixOS/nixpkgs-channels.git; ref = "nixos-unstable"; };

in

{ pkgs? import nixpkgs { config = {}; }}:

let crate2nix = pkgs.callPackage ./default.nix {};
in pkgs.stdenv.mkDerivation {
   name = "shell-with-crate2nix";
   src = ./.;

   buildInputs = [ crate2nix ];

   shellHook = ''
    source ${crate2nix}/share/bash-completion/completions/crate2nix.bash
    crate2nix help
   '';
 }
