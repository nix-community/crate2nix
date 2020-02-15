#!/usr/bin/env bash

# Executes the pinned nix-test-runner.
#
# Example: ./nix-test-runner.sh ./unit-tests.nix

set -Eeuo pipefail

top=$(dirname "$0")

if [ -z "${IN_CRATE2NIX_SHELL:-}" ]; then
  echo "=== Entering $top/shell.nix"
  exec nix-shell --pure "$top/shell.nix" --run "$(printf "%q " $0 "$@")" 
fi

nix-test "$@"