#!/usr/bin/env bash

set -Eeuo pipefail

top="$(readlink -f "$(dirname "$0")")"

if [ -z "${IN_CRATE2NIX_SHELL:-}" ]; then
  exec nix-shell --pure "$top/shell.nix" --run "$(printf "%q " $0 "$@")"
fi

options=$(getopt -o '' --long offline,no-cargo-build -- "$@")
[ $? -eq 0 ] || {
    echo "Incorrect options provided. Available:"
    echo "   --offline Enable offline friendly operations with out substituters"
    echo "   --no-cargo-build     Skip local cargo build." >&2
    exit 1
}
eval set -- "$options"
NIX_OPTIONS="--option log-lines 100 --show-trace"
NO_CARGO_BUILD=""
while true; do
    case "$1" in
    --no-cargo-build)
        NO_CARGO_BUILD=1
        ;;
    --offline)
        NIX_OPTIONS="--option substitute false"
        ;;
    --)
        shift
        break
        ;;
    esac
    shift
done

echo "================ Regenerating ./Cargo.nix =================="

cd "${top}"

function noisily {
  set -x
  "$@"
  { set +x; } 2>/dev/null
  return $?
}

if [ -z "${NO_CARGO_BUILD}" ]; then
  (cd crate2nix; noisily ../cargo.sh run -- generate -n ../nix/nixpkgs.nix \
    -f ./Cargo.toml -o ./Cargo.nix)  ||\
      { echo "Bootstrap regeneration of ./Cargo.nix failed." >&2 ; exit 1; }
else
  echo "Skipping because of --no-cargo-build"
fi

noisily nix-build --arg release false $NIX_OPTIONS
crate2nix=$(nix-build --arg release false $NIX_OPTIONS)/bin/crate2nix
noisily "$crate2nix" generate -n ../nix/nixpkgs.nix \
  -f ./crate2nix/Cargo.toml -o ./crate2nix/Cargo.nix || \
     { echo "Regeneration of ./Cargo.nix failed." >&2 ; exit 1; }

nix-instantiate tests.nix --eval --strict --json -A buildTestConfigs | \
 jq -r .[].pregeneratedBuild | \
 while read cargo_nix; do
   if [ "$cargo_nix" = "null" ]; then
     continue
   fi

   dir=$(dirname "$cargo_nix")

   echo "=============== Regenerating ${cargo_nix} ================"

   noisily "$crate2nix" generate -f "$dir/Cargo.toml" -o "$cargo_nix" ||\
     { echo "Regeneration of ${cargo_nix} failed." >&2 ; exit 1; }
 done
