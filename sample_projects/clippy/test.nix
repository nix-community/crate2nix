{ pkgs ? import ../../nix/nixpkgs.nix { config = { }; }
, generatedCargoNix ? ./Cargo.nix
}:
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
  instantiatedBuild = pkgs.callPackage generatedCargoNix {
    inherit buildRustCrateForPkgs;
  };
in
instantiatedBuild.rootCrate.build.override {
  runClippy = true;
  # Disable the default -Dwarnings to inspect the clippy output in tests.nix
  clippyArgs = [ ];
}
