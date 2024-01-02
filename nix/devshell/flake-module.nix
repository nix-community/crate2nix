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
    , pkgs
    , ...
    } @ perSystem: {
      devshells.default = {
        packages = [
          pkgs.nil
          pkgs.nixpkgs-fmt
          pkgs.pre-commit
        ];

        devshell.startup.pre-commit.text = ''
          ${perSystem.config.pre-commit.installationScript}
        '';
      };
    };
}
