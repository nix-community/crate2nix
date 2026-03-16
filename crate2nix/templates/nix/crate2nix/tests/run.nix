{ pkgs
, nixTestRunner
,
}:
nixTestRunner.runTests {
  tests = pkgs.callPackage ./tests.nix { };
}
