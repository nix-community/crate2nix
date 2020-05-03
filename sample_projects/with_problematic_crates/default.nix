{ pkgs ? import ../../nix/nixpkgs.nix { config = { }; }
, stdenv ? pkgs.stdenv
, generatedCargoNix
}:
let
  generatedBuild = pkgs.callPackage generatedCargoNix {
    defaultCrateOverrides = pkgs.defaultCrateOverrides // {
      pest_generator = attrs: {
        buildInputs =
          (attrs.buildInputs or [ ])
            ++ stdenv.lib.optionals
            stdenv.isDarwin
            (with pkgs.darwin.apple_sdk.frameworks; [ Security ]);
      };

      cssparser-macros = attrs: {
        buildInputs =
          (attrs.buildInputs or [ ])
            ++ stdenv.lib.optionals
            stdenv.isDarwin
            (with pkgs.darwin.apple_sdk.frameworks; [ Security ]);
      };
    };
  };
in
generatedBuild.rootCrate.build
