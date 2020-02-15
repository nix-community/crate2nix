#!/usr/bin/env bash

set -Eeuo pipefail

top="$(readlink -f "$(dirname "$0")")"

if [ -z "${IN_CRATE2NIX_SHELL:-}" ]; then
  export CACHIX="$(which cachix || echo "")"
  echo "=== Entering $top/shell.nix"
  exec nix-shell --keep CACHIX --pure "$top/shell.nix" --run "$(printf "%q " $0 "$@")" 
fi

# Add other files when we adopt nixpkgs-fmt for them.
cd "$top"
./nixpkgs-fmt.sh \
    ./{,nix/}*.nix \
    ./crate2nix/templates/nix/crate2nix/{*.nix,tests/*.nix} \
    ./sample_projects/*/[[:lower:]]*.nix
cd "$top"/crate2nix
./cargo.sh fmt

cd "$top"
./nix-test.sh ./crate2nix/templates/nix/crate2nix/tests/default.nix || {
    echo "" >&2
    echo "==================" >&2
    echo "$top/nix-test.sh $top/crate2nix/templates/nix/crate2nix/tests/default.nix: FAILED" >&2
    exit 1
}

cd "$top"/crate2nix
./cargo.sh clippy || {
    echo "==================" >&2
    echo "$top/crate2nix/cargo.sh clippy: FAILED" >&2
    exit 2
}

../regenerate_cargo_nix.sh || {
    echo "==================" >&2
    echo "$top/regenerate_cargo_nix.sh: FAILED" >&2
    exit 3
}

./cargo.sh test || {
    echo "==================" >&2
    echo "$top/crate2nix/cargo.sh test: FAILED" >&2
    exit 4
}

cd "$top"
nix-build ./tests.nix || {
    echo "==================" >&2
    echo "cd $top; nix-build ./tests.nix: FAILED" >&2
    exit 4
}

# Crude hack: check if we have the right to push to the cache
cd "$top"/crate2nix
if test -n "${CACHIX:-}" && test -r ~/.config/cachix/cachix.dhall &&\
 grep -q '"eigenvalue"' ~/.config/cachix/cachix.dhall; then
    echo "Pushing build artifacts to eigenvalue.cachix.org..." >&2
    # we filter for "rust_" to exclude some things that are in the
    # nixos cache anyways
    nix-store -q -R --include-outputs $(nix-store -q -d target/nix-result*) |\
     grep -e "-rust_" |\
     $CACHIX push eigenvalue
fi