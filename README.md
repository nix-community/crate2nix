# crate2nix

`crate2nix` builds your [cargo](https://crates.io/)-based [rust](https://www.rust-lang.org/) project
crate-by-crate with [nix](https://nixos.org/nix/).

You can

* save time by only rebuilding changed crates hermetically in CI, and
* use `cargo`/`rust-analyzer` locally for a fast developing loop.

➡️ [Read more](https://nix-community.github.io/crate2nix/) ⬅️

[![tests-nix-linux](https://github.com/nix-community/crate2nix/actions/workflows/tests-nix-linux.yml/badge.svg)](https://github.com/nix-community/crate2nix/actions/workflows/tests-nix-linux.yml)
[![tests-nix-macos](https://github.com/nix-community/crate2nix/actions/workflows/tests-nix-macos.yml/badge.svg)](https://github.com/nix-community/crate2nix/actions/workflows/tests-nix-macos.yml)
[![Crate](https://img.shields.io/crates/v/crate2nix.svg)](https://crates.io/crates/crate2nix)
