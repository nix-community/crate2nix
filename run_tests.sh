#!/usr/bin/env bash

top="$(readlink -f "$(dirname "$0")")"

cd "$top"/crate2nix

../regenerate_cargo_nix.sh && nix run nixpkgs.cargo -c cargo test

# Crude hack check if we have the right to push to the cache
if grep -q '"eigenvalue"' ~/.config/cachix/cachix.dhall; then
    echo "Pushing build artifacts to eigenvalue.cachix.org..." >&2
    # we filter for "rust_" to exclude some things that are in the
    # nixos cache anyways
    nix-store -q -R --include-outputs $(nix-store -q -d result*) |\
     grep -e "-rust_" |\
     cachix push eigenvalue
fi