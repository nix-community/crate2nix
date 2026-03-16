{ self, inputs, lib, ... }: {
  flake.overlays.default = final: prev: {
    crate2nix = prev.callPackage ./default.nix { };
  };

  perSystem =
    { pkgs
    , system
    , ...
    }@perSystem: {
      # imports = [
      #   inputs.pre-commit-hooks.flakeModule
      # ];

      config.devshells.default = {
        imports = [
          "${inputs.devshell}/extra/language/c.nix"
          "${inputs.devshell}/extra/language/rust.nix"
        ];

        packages = with pkgs; [
          rust-analyzer
          clippy
          rustc
          rustfmt
        ];

        commands = with pkgs; [
          { package = cargo; category = "rust"; }
        ];

        language.c = {
          libraries = lib.optional pkgs.stdenv.isDarwin pkgs.libiconv;
        };
      };

      config.pre-commit = {
        settings.settings.rust.cargoManifestPath = "crate2nix/Cargo.toml";
        settings.hooks = {
          # rust
          rustfmt.enable = true;
          # clippy.enable = true;
        };
      };

      config.packages.default = pkgs.callPackage ./default.nix { };
      config.packages.crate2nix-from-json = pkgs.callPackage ./default-json.nix { };
      config.checks =
        let
          # Note: This uses the build of the nix-test binary using the stable nixpkgs/crate2nix.
          #       The "unstable" build is tested in the tests.nix checks.
          nixTestRunner = import "${self}/nix/nix-test-runner" { inherit system; };
        in
        {
          unit-tests = pkgs.callPackage ./templates/nix/crate2nix/tests/run.nix {
            inherit nixTestRunner;
          };
        } // (pkgs.callPackage ../tests.nix { }).checks;
    };
}
