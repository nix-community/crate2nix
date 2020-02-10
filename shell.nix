#
# Crates a shell with crate2nix installed.
#

# Use pinned version of nixos-unstable by default to make sure this works.
# Override with e.g.
#    nix-shell --arg pkgs 'import <nixos> {config = {}; }'
{pkgs? import ./nixpkgs.nix { config = {}; }}:

let crate2nix = pkgs.callPackage ./default.nix {};
in pkgs.stdenv.mkDerivation {
   name = "shell-with-crate2nix";

   buildInputs = [ 
     crate2nix 
     pkgs.nixpkgs-fmt
   ];

   shellHook = ''
    echo "" >&2
    echo -e "\e[1mDeprecation\e[0m:" >&2
    echo -e "  \e[1mcrate2nix\e[0m will soon not be installed by shell.nix (this environment) anymore." >&2
    echo -e "  Please refer to the README.md for other installation methods." >&2
    echo "" >&2

    source ${crate2nix}/share/bash-completion/completions/crate2nix.bash
    crate2nix help
   '';
 }
