---
title: Rust Project via Flake Template
description: How to create a new rust project using crate2nix via flakes.
sidebar:
  badge: { text: 'Experimental', variant: 'caution' }
---

* *️⃣ Uses [nix flakes](https://zero-to-nix.com/concepts/flakes).
* ✅ No need to install `crate2nix`.
* ✅ Auto-generates nix from your `Cargo.lock` file.
* ⚠️ Uses the [import from derivation](https://nixos.org/manual/nix/stable/language/import-from-derivation)
  feature from Nix which may lead to disabling build parallelism.
* ✅ Uses an overlay to use the rust version specified in `rust-toolchain.toml`.

Call this in your project directory:

```bash title="Add flake.nix and other files to your project"
nix flake init --template github:nix-community/crate2nix
```

If the directory is empty, it will also create a hello world stub.

If you call this from an existing project, make sure to delete any superfluous
files. The template expects top-level `Cargo.toml`/`Cargo.lock` files,
otherwise you need to adjust `flake.nix` manually.

```bash title="Building project and running tests"
nix build
```

```bash title="Building & running project"
nix run
```

```bash title="Building & running project with arguments"
nix run -- --arg1 x
```
