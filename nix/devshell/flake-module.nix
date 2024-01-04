{ inputs
, self
, ...
}: {
  imports = [
    inputs.devshell.flakeModule
  ];
  config.perSystem =
    { config
    , system
    , lib
    , pkgs
    , ...
    } @ perSystem: {
      devshells.default = {
        imports = [
          "${inputs.devshell}/extra/language/c.nix"
          "${inputs.devshell}/extra/language/rust.nix"
        ];

        packages = with pkgs; [
          nil
          nixpkgs-fmt
          pre-commit

          cargo
          clippy
          rustc
          rustfmt
          nixpkgs-fmt
          jq
          nix
          niv
          git
          coreutils
          gnugrep
          utillinux
          cacert
          nix-prefetch-git
          (import ../nix-test-runner.nix { inherit pkgs; })
        ];

        language.c = {
          libraries = lib.optional pkgs.stdenv.isDarwin pkgs.libiconv;
        };

        env = [
          {
            name = "IN_CRATE2NIX_SHELL";
            value = "1";
          }
          {
            name = "NIX_PATH";
            value =
              # TODO: Substitute sources with flake inputs
              let sources = import ../../nix/sources.nix;
              in "nixpkgs=${sources.nixpkgs}";
          }
        ];

        devshell.startup.pre-commit.text = ''
          ${perSystem.config.pre-commit.installationScript}
        '';
      };
    };
}
