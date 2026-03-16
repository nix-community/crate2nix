---
title: Running clippy
sidebar:
  badge: { text: 'Experimental', variant: 'caution' }
---

Much the same way that tests can be run via `crate2nix`, you can also have it run clippy on your
crates.

## Running clippy

There is some experimental support for running clippy against your rust crates. When clippy is
enabled (`runCippy = true;`), an additional derivation will be built which replaces `rustc` with
`clippy-driver` for your crate. For this, you'll need to provide a toolchain containing both `rustc`
and `clippy-driver`. You can link one together from the `rustc` and `clippy` packages in `nixpkgs`,
or [fenix] is a good option. See [the section on
toolchains](../../35_toolchains/10_custom_toolchains/) for more details.

```nix
  let
    buildRustCrateForPkgs = pkgs: pkgs.buildRustCrate.override {
      rustc = pkgs.symlinkJoin {
        name = "rustc-with-clippy";
        paths = [
          pkgs.rustc
          pkgs.clippy
        ];
      };
    };

    cargoNix = pkgs.callPackage ./Cargo.nix {
      inherit buildRustCrateForPkgs;
    };
    
    crate2nix = cargo_nix.rootCrate.build.override {
      runClippy = true;
      clippyArgs = [ ... ];
    };
  in ...
```

If omitted, `clippyArgs` contains `-Dwarnings` by default, which will turn any warnings into errors.

### Additional notes

* While your crates are rebuilt using `clippy-driver`, your dependencies will continue to be built
  with `rustc`, allowing them to be reused.
* Clippy will run against all crate targets, including tests and integration tests.

[fenix]: https://github.com/nix-community/fenix
