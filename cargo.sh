#!/usr/bin/env bash

# Executes cargo from the pinned nixpkgs
#
# Example: ./cargo.sh test

set -Eeuo pipefail

top="$(dirname "$0")/.."
top="$(cd "$top"; pwd)"

if [ -z "${IN_CRATE2NIX_SHELL:-}" ]; then
  echo "=== Entering $top/shell.nix"
  exec nix-shell --pure "$top/shell.nix" --run "$(printf "%q " $0 "$@")"
fi

export TEMPLATES_DIR="$top/crate2nix/templates"

cargo "$@"
