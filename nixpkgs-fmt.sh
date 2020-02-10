#!/usr/bin/env bash

# Executes nixpkgs-fmt from the pinned nixpkgs
#
# Example: ./nixpkgs-fmt.sh ./tests.nix

set -Eeuo pipefail

mydir=$(dirname "$0")

nix run "(import $mydir/nixpkgs.nix { config = {}; }).nixpkgs-fmt" -c nixpkgs-fmt "$@"
