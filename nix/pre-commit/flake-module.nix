{ inputs, ... }: {
  imports = [
    inputs.pre-commit-hooks.flakeModule
  ];
  config.perSystem =
    { config
    , system
    , pkgs
    , ...
    } @ perSystem: {
      # https://github.com/cachix/pre-commit-hooks.nix/tree/master
      pre-commit = {
        check.enable = true;

        settings.hooks = {
          # # lint shell scripts
          # # TODO: https://github.com/NixOS/nix/issues/8761
          # shellcheck.enable = true;
          # # nix format
          # # TODO: need to preformat things accordingly and potentially use another formatter
          # nixpkgs-fmt.enable = true;
        };
      };
    };
}
