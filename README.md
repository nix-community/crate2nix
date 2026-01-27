# crate2nix

[![tests-nix-linux](https://github.com/nix-community/crate2nix/actions/workflows/tests-nix-linux.yml/badge.svg)](https://github.com/nix-community/crate2nix/actions/workflows/tests-nix-linux.yml)
[![tests-nix-macos](https://github.com/nix-community/crate2nix/actions/workflows/tests-nix-macos.yml/badge.svg)](https://github.com/nix-community/crate2nix/actions/workflows/tests-nix-macos.yml)
[![Crate](https://img.shields.io/crates/v/crate2nix.svg)](https://crates.io/crates/crate2nix)

`crate2nix` generates [Nix](https://nixos.org/nix/) build files for
[Rust](https://www.rust-lang.org/)/[Cargo](https://crates.io/) projects,
building each crate individually for precise, incremental rebuilds.

- **Incremental CI builds** -- only rebuild the crates that actually changed.
- **Full Nix integration** -- remote builds, binary caches, Docker images, NixOS modules.
- **Local dev unchanged** -- keep using `cargo` and `rust-analyzer` as usual.

## Quick start

### Without installing

```bash
nix run nixpkgs#crate2nix -- generate
nix build -f Cargo.nix rootCrate.build
```

### With a flake template

```bash
nix flake init --template github:nix-community/crate2nix
```

### Installing

```bash
# From nixpkgs
nix profile install nixpkgs#crate2nix

# Latest development version
nix profile install github:nix-community/crate2nix
```

Then, inside your project:

```bash
crate2nix generate        # creates Cargo.nix
nix build -f Cargo.nix rootCrate.build
```

## How it works

`crate2nix` reads `Cargo.toml` and `Cargo.lock`, resolves the full dependency
tree via `cargo metadata`, prefetches source hashes, and renders a `Cargo.nix`
file through Tera templates. The generated file contains one Nix derivation per
crate, so Nix rebuilds only what changed.

Two generation strategies are supported:

| Strategy | Pros | Cons |
| --- | --- | --- |
| **Manual** (`crate2nix generate`) | No IFD, full build parallelism | Must regenerate when deps change |
| **Auto** (Import From Derivation) | Always in sync with `Cargo.lock` | May reduce parallelism |

## Nix API

`tools.nix` exposes helpers for use in your own Nix expressions:

```nix
let
  crate2nix = builtins.fetchTarball "https://github.com/nix-community/crate2nix/tarball/master";
  tools = import "${crate2nix}/tools.nix" { inherit pkgs; };

  generated = tools.generatedCargoNix {
    name = "my-project";
    src = ./.;
  };

  project = pkgs.callPackage "${generated}/default.nix" {};
in
  project.rootCrate.build
```

Or the shorthand `appliedCargoNix` which combines generation and import.

## Documentation

Full documentation is at
**<https://nix-community.github.io/crate2nix/>**, covering:

- [Installation options](https://nix-community.github.io/crate2nix/10_getting_started/20_installing_crate2nix/)
- [Generation strategies](https://nix-community.github.io/crate2nix/20_generating/10_generating/)
- [Building binaries](https://nix-community.github.io/crate2nix/30_building/10_building_binaries/)
- [Feature selection](https://nix-community.github.io/crate2nix/30_building/20_choosing_features/)
- [Crate overrides](https://nix-community.github.io/crate2nix/30_building/30_crateoverrides/)
- [Known restrictions](https://nix-community.github.io/crate2nix/90_reference/20_known_restrictions/)
- [Changelog](https://nix-community.github.io/crate2nix/90_reference/90_changelog/)

## Contributing

Contributions are welcome! See the
[contributing guide](https://nix-community.github.io/crate2nix/50_contributing/)
for details.

## License

Apache-2.0
