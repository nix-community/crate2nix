{ pkgs ? import ../../nix/nixpkgs.nix { config = { }; }
, generatedCargoNix ? ./Cargo.nix { }
}:

let pkgs0 = pkgs; in

let
  pkgs = import pkgs0.path {
    config = { };
    crossSystem = {
      config = "riscv32-unknown-none-elf";
      rustc = {
        config = "riscv32i-unknown-none-elf";
      };
    };
  };
  inherit (pkgs) lib;

  instantiatedBuild = pkgs.callPackage generatedCargoNix {
    defaultCrateOverrides = pkgs.defaultCrateOverrides;
    buildRustCrateForPkgs = pkgs:
      let
        isBareMetal = pkgs.stdenv.hostPlatform.parsed.kernel.name == "none";
        # Don't need other tools
        stdenvBase = if isBareMetal then pkgs.stdenvNoCC else pkgs.stdenv;

        stdenv =
          if stdenvBase.hostPlatform.extensions ? sharedLibrary
          then stdenvBase
          else
            lib.recursiveUpdate stdenvBase {
              # This is used in buildRustCrate. Should probably be optional there.
              hostPlatform.extensions.sharedLibrary = "";
            };

        fun = pkgs.buildRustCrate.override {
          inherit stdenv;

          # Don't bother with cross compiler since we don't need stdlib
          inherit (pkgs.buildPackages.buildPackages) rust rustc cargo;
        };
      in
      args: fun (args // lib.optionalAttrs isBareMetal {
        RUSTC_BOOTSTRAP = true;

        # Needed for std for build.rs
        nativeBuildInputs = args.nativeBuildInputs or [ ]
          ++ lib.optional pkgs.stdenv.buildPlatform.isDarwin pkgs.buildPackages.libiconv;
      });
  };
in
instantiatedBuild.rootCrate.build
