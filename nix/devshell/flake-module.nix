{ inputs
, ...
}: {
  imports = [
    inputs.devshell.flakeModule
  ];
  config.perSystem =
    { config
    , system
    , inputs'
    , lib
    , pkgs
    , ...
    } @ perSystem: {
      devshells.default = {
        packages = with pkgs; [
          nil

          jq
          niv
          coreutils
          gnugrep
          utillinux
          cacert
        ];

        commands = with pkgs; [
          { package = gitMinimal; }
          { package = pre-commit; }
          { package = nixpkgs-fmt; category = "nix"; }
          { package = nix; category = "nix"; }
          { package = nix-prefetch-git; category = "nix"; }
          {
            name = "nix-test";
            package = import ../nix-test-runner.nix { inherit (pkgs) system; };
            category = "nix";
            help = "nix test runner for unit tests.";
          }
          { package = inputs'.cachix.packages.default; category = "nix"; }
        ];

        env = [
          {
            name = "IN_CRATE2NIX_SHELL";
            value = "1";
          }
          {
            name = "NIX_PATH";
            value = "nixpkgs=${inputs.nixpkgs}";
          }
        ];

        devshell.startup.pre-commit.text = ''
          ${perSystem.config.pre-commit.installationScript}
        '';
      };
    };
}
