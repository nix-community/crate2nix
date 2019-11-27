#!/usr/bin/env bash

top="$(readlink -f "$(dirname "$0")")"

cd "$top"/crate2nix

../regenerate_cargo_nix.sh && nix run nixpkgs.cargo -c cargo test
