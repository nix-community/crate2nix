{ self, inputs, lib, ... }: {
  flake.overlays.default = final: prev: {
    crate2nix = prev.callPackage ./default.nix { };
  };

  perSystem =
    { pkgs
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
        settings.package = pkgs.prek;
        settings.settings.rust.cargoManifestPath = "crate2nix/Cargo.toml";
        settings.hooks = {
          # rust
          rustfmt.enable = true;
          # clippy.enable = true;
        };
      };

      config.packages.default = pkgs.callPackage ./default.nix { };
    };
}
