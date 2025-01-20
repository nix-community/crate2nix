---
title: Building fetched sources
---

* ✅ No need to install `crate2nix`.
* ✅ Auto-generates nix from your `Cargo.lock` file.
* ⚠️ Uses the [import from derivation](https://nixos.org/manual/nix/stable/language/import-from-derivation)
  feature from Nix which may lead to disabling build parallelism.

Do you want to build a rust binary but you cannot simply add the necessary nix-files
to the source repository? You don't need to.

The `crate2nix` repo itself contains an example where it builds an external repository
using the [tools.nix](./31_auto_generating) support:

```nix
# nix/nix-test-runner.nix
let
  # Reuses the locked flake inputs.
  flakeLock = builtins.fromJSON (builtins.readFile ../flake.lock);
  # Gets the locked sources.
  src = builtins.fetchTree flakeLock.nodes.nix-test-runner.locked;
in
{ pkgs ? import ./nixpkgs.nix { }
  # Use last pinned crate2nix packages to build the test runner
  # so that it works even if we have broken stuff!
, tools ? pkgs.callPackage "${builtins.fetchTree flakeLock.nodes.crate2nix_stable.locked}/tools.nix" { }
}:
let
  nixTestRunner = tools.appliedCargoNix {
    name = "nix-test-runner";
    inherit src;
  };
in
nixTestRunner.rootCrate.build
```

```nix
# nix/nixpkgs.nix
let
  flakeLock = builtins.fromJSON (builtins.readFile ../flake.lock);
in
import "${builtins.fetchTree flakeLock.nodes.nixpkgs.locked}"
```

```nix
# flake.nix
{
  # ...
  inputs = {
    nixpkgs.url = "nixpkgs";
    # ...

    crate2nix_stable = {
      url = "github:nix-community/crate2nix/0.12.0";
    };

    nix-test-runner = {
      url = "github:stoeffel/nix-test-runner";
      flake = false;
    };
  };

  # ...
}
```
