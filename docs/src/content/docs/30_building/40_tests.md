---
title: Rust tests
sidebar:
  badge: { text: 'Experimental', variant: 'caution' }
---

:::tip[Not thaaat experimental]

Running tests has been supported by `crate2nix` for a while now but is still
marked as experimental. Don't let that scare you too much: we intend to support
tests in the future but the interface might slightly change.

:::

## Running rust tests

There is some experimental support for running tests of your rust crates. All of
the crates in the workspace will have their tests executed. When enabling test
execution (`runTests = true;`), failing tests will make the whole build fail
unless you explicitly disable this via test hooks: see the section below.

```nix
      let cargo_nix = callPackage ./Cargo.nix {};
          crate2nix = cargo_nix.rootCrate.build.override {
     runTests = true;
     testInputs = [ pkgs.cowsay ];
   };
      in ...
```

`testInputs` is optional and allows passing inputs to the test execution that
should be in scope. Defaults to an empty list and is ignored when `runTests`
equals `false`.

## Custom pre/post test hooks

Want to customize your test execution? Use the `testPreRun` and `testPostRun`
crate attributes(next to `runTests` in the example above). `crate2nix` executes
the bash snippets in `testPreRun` and `testPostRun` directly before and after
the actual test command, and in the same shell. Some example use-cases include:

* Setting some environment variable that is needed for the test.

* Setting (and then unsetting) the bash `set +e` option to not fail the
  derivation build even if a test fails. This is quite useful if your tests are
  not flaky and you want to cache failures.
