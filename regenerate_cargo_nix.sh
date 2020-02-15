#!/usr/bin/env bash

set -Eeuo pipefail

top="$(readlink -f "$(dirname "$0")")"

if [ -z "${IN_CRATE2NIX_SHELL:-}" ]; then
  exec nix-shell --pure "$top/shell.nix" --run "$(printf "%q " $0 "$@")" 
fi

echo "================ Regenerating ./Cargo.nix =================="

cd "${top}"

(cd crate2nix; ./cargo.sh run -- generate -n ../nix/nixpkgs.nix \
  -f ./Cargo.toml -o ./Cargo.nix)  ||\
     { echo "Bootstrap regeneration of ./Cargo.nix failed." >&2 ; exit 1; }

nix run -c crate2nix generate -n ../nix/nixpkgs.nix \
  -f ./crate2nix/Cargo.toml -o ./crate2nix/Cargo.nix || \
     { echo "Regeneration of ./Cargo.nix failed." >&2 ; exit 1; }

nix build

crate2nix="$(pwd)"/result/bin/crate2nix

nix eval --json -f ./tests.nix buildTestConfigs | \
 jq -r .[].pregeneratedBuild | \
 while read cargo_nix; do
   if [ "$cargo_nix" = "null" ]; then
     continue
   fi

   dir=$(dirname "$cargo_nix")

   echo "=============== Regenerating ${cargo_nix} ================"

   "$crate2nix" generate -f "$dir/Cargo.toml" -o "$cargo_nix" ||\
     { echo "Regeneration of ${cargo_nix} failed." >&2 ; exit 1; }
 done
