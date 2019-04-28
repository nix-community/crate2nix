#!/usr/bin/env bash

cargo run -- "generate" "-n" "./nixpkgs.nix" "-o" "./Cargo.nix"
