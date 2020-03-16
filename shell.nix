#
# A development shell environment for crate2nix.
#
# This can be used for interactive development.
# A pure version of this is used for "run_tests.sh" and "regenerate_cargo.sh".
#

# Use pinned version of nixos-unstable by default to make sure this works.
# Override with e.g.
#    nix-shell --arg pkgs 'import <nixos> {config = {}; }'
{ pkgs ? import ./nix/nixpkgs.nix { config = {}; }
, sources ? import ./nix/sources.nix
, dependencies ? pkgs.callPackage ./nix/dependencies.nix {}
, lib ? pkgs.lib
}:

pkgs.mkShell {
  buildInputs = lib.attrValues dependencies.dev;

  shellHook = ''
    if [ -n "$PS1" ]; then
      # We are in an interactive shell
      echo "" >&2
      echo -e "\e[1mNote\e[0m:" >&2
      echo -e "  \e[1mcrate2nix\e[0m is not nstalled by shell.nix (this environment) anymore." >&2
      echo -e "  Please refer to the README.md for other installation methods." >&2
      echo "" >&2
    fi

    export NIX_PATH="nixpkgs=${sources.nixpkgs}"
    export IN_CRATE2NIX_SHELL=1;
  '';
}
