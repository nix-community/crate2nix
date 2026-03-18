let
  # Use last stable crate2nix release to build the test runner so that it
  # works even if the current branch has broken stuff.
  # Pinned directly instead of via flake input to avoid bloating the
  # dependency graph (see https://github.com/nix-community/crate2nix/issues/371).
  crate2nix_stable = builtins.fetchTree {
    type = "github";
    owner = "nix-community";
    repo = "crate2nix";
    rev = "7c33e664668faecf7655fa53861d7a80c9e464a2"; # 0.15.0
    narHash = "sha256-SUuruvw1/moNzCZosHaa60QMTL+L9huWdsCBN6XZIic=";
  };
  nixpkgs_stable = builtins.fetchTree {
    type = "github";
    owner = "NixOS";
    repo = "nixpkgs";
    rev = "addf7cf5f383a3101ecfba091b98d0a1263dc9b8"; # crate2nix 0.15.0's nixpkgs
    narHash = "sha256-hM20uyap1a0M9d344I692r+ik4gTMyj60cQWO+hAYP8=";
  };
in
{ system ? builtins.currentSystem
, pkgs ? import nixpkgs_stable { inherit system; }
, crate2nixTools ? pkgs.callPackage "${crate2nix_stable}/tools.nix" { }
, nix-test-runner ? pkgs.callPackage ./package.nix {
    inherit crate2nixTools;
  }
, lib ? pkgs.lib
}:

rec {
  package = nix-test-runner;

  /* Runs your nix tests from a file or an expression
     and outputs a pretty diff if they fail.

     Type: runTests attrSet -> derivation

     Example:
      runTests { testFile = ./examples/failing.nix; }
      => returns a derivation that will show a failure diff.
      runTests {
        tests = {
          testFailed = {
            expr = builtins.add 1 1;
            expected = 1;
          };
        };
      }
      => returns a derivation that will show a failure diff.

    You need to pass one of the following arguments:

      testFile - the nix file to import that evaluates to the nix expressions.
      tests    - the nix expression containing the tests. Takes precedence.

    Optional arguments:

      name         - used in the derivation(s) produced (for the test results as
                     JSON etc.).
      alwaysPretty - also print pretty results for passing tests.

    If there are no failures, returns a derivation with an empty output.
   */
  runTests =
    { name ? if testFile != null
      then "nix-tests-${builtins.baseNameOf testFile}"
      else "nix-tests"
    , testFile ? null
    , tests ? import testFile
    , alwaysPretty ? false
    }:
    let
      result = testResult { inherit tests lib; };
      debugTestOrigin =
        if testFile != null
        then "${name} imported from ${toString testFile}"
        else name;
      resultJson = pkgs.writeTextFile {
        name = "${name}-result.json";
        text = builtins.toJSON result;
      };
      failed = result.failed or [ ];
      allGood = failed == [ ];
    in
    if allGood
    then
      (
        if alwaysPretty
        then
          pkgs.runCommandLocal "${name}-passed" { }
            ''
              echo -e "\e[1mPASSED\e[0m: ${debugTestOrigin}"
              touch $out
            ''
        else
          pkgs.runCommandLocal "${name}-passed" { }
            ''
              echo -e "\e[1mPASSED\e[0m: ${debugTestOrigin}"
              echo ""
              (
                set -x
                ${nix-test-runner}/bin/nix-test --skip-run ${resultJson} | tee $out
              )
            ''
      )
    else
      pkgs.runCommandLocal
        "${name}-failed"
        { }
        ''
          echo -e "\e[1m\e[31mFAILED\e[0m: ${debugTestOrigin}"
          echo ""
          (
            set -x
            ${nix-test-runner}/bin/nix-test --skip-run ${resultJson}
          )
        '';

  /* Returns the prettified test results as processed by nix-test-runner. */
  testResult = import ./runTest.nix;
}
