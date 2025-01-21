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
  lib = import ./lib.nix;
  # Gets the locked test runner source from flake inputs.
  src = builtins.fetchTree (lib.flakeInput "nix-test-runner");
in
{ system ? null # a system must be specified if using default value for pkgs

  # Use last pinned crate2nix packages and corresponding nixpkgs to build the
  # test runner so that it works even if we have broken stuff!
, pkgs ? import (builtins.fetchTree (lib.flakeNestedInput [ "crate2nix_stable" "nixpkgs" ])) { inherit system; }
, tools ? pkgs.callPackage "${builtins.fetchTree (lib.flakeInput "crate2nix_stable")}/tools.nix" { }
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
  flakeInput = (import ./lib.nix).flakeInput;
in
import (builtins.fetchTree (flakeInput "nixpkgs"))
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
