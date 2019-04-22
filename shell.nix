#
# Crates a shell with crate2nix installed.
#

# Use pinned version of nixos-unstable by default to make sure this works.
# Override with e.g.
#    nix-shell --arg pkgs 'import <nixos> {config = {}; }'
{pkgs? import ./nixpkgs.nix}:

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
