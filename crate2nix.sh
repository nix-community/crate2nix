#!/usr/bin/env bash

# Runs crate2nix as defined in this repo with pinned nixpkgs.
#
# Crate2nix is rebuilt if necessary.

set -Eeuo pipefail

mydir=$(dirname "$0")

nix run "${mydir}" -- "$@"