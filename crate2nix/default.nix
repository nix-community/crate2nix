# Provided by callPackage or also directly usable via nix-build with defaults.
{ pkgs ? (
    import (builtins.fetchTree (import ../nix/flakeInput.nix "nixpkgs")) { }
  )
, stdenv ? pkgs.stdenv
, lib ? pkgs.lib
, symlinkJoin ? pkgs.symlinkJoin
, makeWrapper ? pkgs.makeWrapper
, darwin ? pkgs.darwin
, defaultCrateOverrides ? pkgs.defaultCrateOverrides
, nix ? pkgs.nix
, cargo ? pkgs.cargo
, libsecret ? pkgs.libsecret
, callPackage ? pkgs.callPackage
, nix-prefetch-git ? pkgs.nix-prefetch-git
  # Seperate arguements that are NOT filled by callPackage.
, cargoNixPath ? ./Cargo.nix
, release ? true
}:
let
  cargoNix = callPackage cargoNixPath {
    inherit release;
    buildRustCrateForPkgs = pkgs: pkgs.buildRustCrate.override {
      rustc = pkgs.symlinkJoin {
        name = "rustc";
        paths = [
          pkgs.rustc
          pkgs.clippy
        ];
      };
    };
  };
  withoutTemplates = name: type:
    let
      baseName = builtins.baseNameOf (builtins.toString name);
    in
      !(baseName == "templates" && type == "directory");
  rootCrate = cargoNix.rootCrate.build.override {
    testCrateFlags = [
      "--skip up_to_date"
    ];
    crateOverrides = defaultCrateOverrides // {
      crate2nix = { src, ... }: {
        src =
          if release
          then src
          else
            lib.cleanSourceWith {
              filter = withoutTemplates;
              inherit src;
            };
        dontFixup = !release;
      };
      cssparser-macros = attrs: assert builtins.trace "cssparser" true;{
        buildInputs = lib.optionals stdenv.isDarwin [ darwin.apple_sdk.frameworks.Security ];
      };
    };
  };
in
import ./mk-crate2nix.nix {
  inherit pkgs stdenv lib symlinkJoin makeWrapper nix cargo libsecret
    nix-prefetch-git release rootCrate;
}
