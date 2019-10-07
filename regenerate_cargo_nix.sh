#!/usr/bin/env bash

cargo run -- "generate" "-n" "./nixpkgs.nix" "-o" "./Cargo.nix"

set -x

nix eval --json -f ./tests.nix buildTestConfigs |\
 jq -r .[].pregeneratedBuild |\
 while read cargo_nix; do
   if [ "$cargo_nix" = "null" ]; then
     continue
   fi

   dir=$(dirname "$cargo_nix")

   cargo run -- "generate" -f "$dir/Cargo.toml" -o "$cargo_nix"
 done
