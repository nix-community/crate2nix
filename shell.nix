#
# Crates a shell with crate2nix installed.
#

# Use pinned version of nixos-unstable by default to make sure this works.
# Override with e.g.
#    nix-shell --arg pkgs 'import <nixos> {config = {}; }'
let nixpkgs = builtins.fetchTarball {
    name = "nixos-unstable-2019-04-21";
    url = https://github.com/nixos/nixpkgs/archive/1fc591f9a5bd1b016b5d66dfab29560073955a14.tar.gz;
    sha256 = "1ij5x1qw486rbih7xh2c01s60c3zblj6ad1isf5y99sh47jcq76c";
};

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
