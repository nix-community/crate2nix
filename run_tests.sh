#!/usr/bin/env bash

set -Eeuo pipefail

top="$(readlink -f "$(dirname "$0")")"

cd "$top"/crate2nix
./cargo.sh fmt
./cargo.sh clippy

../regenerate_cargo_nix.sh && ./cargo.sh test || {
    echo "==================" >&2
    echo "cargo test: FAILED" >&2
    exit 1
}

# Add other files when we adopt nixpkgs-fmt for them.
cd "$top"
./nixpkgs-fmt.sh \
    ./{tests,tools}.nix \
    ./crate2nix/templates/nix/crate2nix/{*.nix,tests/*.nix} \
    ./sample_projects/*/*.nix

# Crude hack: check if we have the right to push to the cache
cd "$top"/crate2nix
if test -r ~/.config/cachix/cachix.dhall &&\
 grep -q '"eigenvalue"' ~/.config/cachix/cachix.dhall; then
    echo "Pushing build artifacts to eigenvalue.cachix.org..." >&2
    # we filter for "rust_" to exclude some things that are in the
    # nixos cache anyways
    nix-store -q -R --include-outputs $(nix-store -q -d target/nix-result*) |\
     grep -e "-rust_" |\
     cachix push eigenvalue
fi