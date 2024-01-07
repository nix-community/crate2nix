{
  description = ''
    crate2nix generates [nix](https://nixos.org/nix/) build files for [rust](https://www.rust-lang.org/) 
    crates using [cargo](https://crates.io/).
  '';

  nixConfig = {
    extra-trusted-public-keys = "eigenvalue.cachix.org-1:ykerQDDa55PGxU25CETy9wF6uVDpadGGXYrFNJA3TUs=";
    extra-substituters = "https://eigenvalue.cachix.org";
    allow-import-from-derivation = true;
  };

  inputs = {
    nixpkgs.url = "nixpkgs";

    flake-compat.url = "https://flakehub.com/f/edolstra/flake-compat/1.tar.gz";

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    # Development

    devshell = {
      url = "github:numtide/devshell";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    crate2nix_stable = {
      url = "github:nix-community/crate2nix/0.12.0";
    };

    nix-test-runner = {
      url = "github:stoeffel/nix-test-runner";
      flake = false;
    };

    cachix = {
      url = "github:cachix/cachix/latest";
      # we only want the binary, so we don't do this
      # inputs.nixpkgs.follows = "nixpkgs";
      inputs.devenv.follows = "";
      inputs.flake-compat.follows = "";
      inputs.pre-commit-hooks.follows = "";
    };

    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.nixpkgs-stable.follows = "nixpkgs";
      inputs.flake-compat.follows = "flake-compat";
    };
  };

  outputs = inputs@{ nixpkgs, self, flake-parts, devshell, pre-commit-hooks, ... }: flake-parts.lib.mkFlake { inherit inputs; } {
    systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];

    imports = [
      ./nix/devshell/flake-module.nix
      ./nix/pre-commit/flake-module.nix
      ./nix/perSystem-tools/flake-module.nix
      ./crate2nix/flake-module.nix
      ./docs/flake-module.nix
    ];

    flake = { lib, ... }: {
      config.templates = rec {
        default = flake-binary;
        flake-binary = {
          path = ./templates/flake-binary;
          description = "An example of crate2nix";
        };
      };

      config.lib = {
        tools = import ./tools.nix;
      };

      options.lib = lib.mkOption {
        description = ''
          nix libraries exported by crate2nix.
        '';

        type = lib.types.submoduleWith {
          modules = [
            {
              options.tools = lib.mkOption {
                description = ''
                  Prefer the perSystem "tools" option which has the libary
                  already applied for the correct system.

                  Export the crate2nix/tools.nix function as property.

                  To use it, call it with pkgs.callPackage.
                '';

                type = lib.types.functionTo lib.types.attrs;
              };
            }
          ];
        };
      };
    };

    perSystem = { config, pkgs, ... }: {
      formatter = pkgs.nixpkgs-fmt;
      checks = config.packages;
    };
  };
}
