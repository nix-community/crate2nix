{ pkgs? import ../../nixpkgs.nix { config = {}; }
, generatedBuild ? ./Cargo.nix { } }:

let instantiatedBuild = pkgs.callPackage generatedBuild {};
in instantiatedBuild.rootCrate.build.override {
    runTests = true;
    testCrateFlags = [
      "--skip" "this_must_be_skipped"
    ];
}
