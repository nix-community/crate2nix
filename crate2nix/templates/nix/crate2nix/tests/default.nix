let
  pkgs = import ../../../../../nix/nixpkgs.nix { };
  lib = pkgs.lib;
  crate2nix = pkgs.callPackage ../default.nix { };
  testFiles = [
    "dependencyDerivations"
    "dependencyFeatures"
    "enableFeatures"
    "expandFeatures"
    "packageFeatures"
  ];
  testsInFile = f:
    let
      tests = (pkgs.callPackage (./. + "/${f}.nix")) { inherit crate2nix; };
      prefixedTests = lib.mapAttrs' (n: v: lib.nameValuePair "${n} in ${f}.nix" (if builtins.isAttrs v then v else { })) tests;
    in
    assert builtins.isAttrs prefixedTests;

    prefixedTests;
  all = lib.foldl (cum: f: cum // (testsInFile f)) { } testFiles;
in
all
