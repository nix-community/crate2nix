---
title: Generate Cargo.nix manually
---

* ✅ Does not use [import from derivation](https://nixos.org/manual/nix/stable/language/import-from-derivation)
  feature from Nix which may lead to disabling build parallelism,
* 🛠️ but you need to regenerate `Cargo.nix` whenever you change your dependencies or other config
  affecting `Cargo.lock`.

Disregarding whether you use nix with flakes or without, you might want to generate
your `Cargo.nix` file manually. That way you avoid using the
[import from derivation](https://nixos.org/manual/nix/stable/language/import-from-derivation)
feature from nix which might disable build parallelism.

Here is a simple example which uses all the defaults and will generate a `Cargo.nix` file.

If you [installed crate2nix locally](../20_installing_crate2nix/):

```bash
# From the project directory with the Cargo.toml.
crate2nix generate
```

If you prefer to run it without installation:

```bash
# From the project directory with the Cargo.toml.
nix run nixpkgs#crate2nix -- generate
```

Here is a more elaborate example that uses `<nixos-unstable>` as the default `pkgs` path
(instead of `<nixpkgs>`) and specifies both the path
to the `Cargo.toml` file (`-f`) and the output (`-o`) file explicitly (usually not needed).

```bash
crate2nix generate \
    -n '<nixos-unstable>' \
    -f /some/project/dir/Cargo.toml \
    -o /some/project/dir/Cargo.nix
```

Use `crate2nix help` to show all commands and options.

Look at the
[./crate2nix/Cargo.nix](https://github.com/nix-community/crate2nix/blob/master/crate2nix/Cargo.nix)
file of this project for a non-trivial example. (How meta!)
