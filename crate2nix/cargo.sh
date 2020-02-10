#!/usr/bin/env bash

# Executes cargo from the pinned nixpkgs
#
# Example: ./cargo.sh test

set -Eeuo pipefail

mydir=$(dirname "$0")

nix run "(with import $mydir/../nixpkgs.nix { config = {}; }; [ cargo clippy rustfmt binutils ])" -c cargo "$@"
