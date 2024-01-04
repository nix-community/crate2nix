{
  description = ''
    crate2nix generates [nix](https://nixos.org/nix/) build files for [rust](https://www.rust-lang.org/) 
    crates using [cargo](https://crates.io/).
  '';

  nixConfig = {
    extra-trusted-public-keys = "eigenvalue.cachix.org-1:ykerQDDa55PGxU25CETy9wF6uVDpadGGXYrFNJA3TUs=";
    extra-substituters = "https://eigenvalue.cachix.org";
  };

  inputs = {
    # TODO: Remove nixpkgs pin after solving https://github.com/nix-community/crate2nix/issues/319
    nixpkgs.url = "nixpkgs/0cbe9f69c234a7700596e943bfae7ef27a31b735";

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
      inputs.nixpkgs.follows = "nixpkgs";
    };

    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.nixpkgs-stable.follows = "nixpkgs";
    };
  };

  outputs = inputs@{ nixpkgs, self, flake-parts, devshell, pre-commit-hooks, ... }: flake-parts.lib.mkFlake { inherit inputs; } {
    systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];

    imports = [
      ./nix/devshell/flake-module.nix
      ./nix/pre-commit/flake-module.nix
      ./crate2nix/flake-module.nix
    ];

    flake = {
      templates.default = {
        path = ./template;
        description = "An example of crate2nix";
      };
    };

    perSystem = { config, self', inputs', pkgs, system, ... }: {
      formatter = pkgs.nixpkgs-fmt;
      checks = config.packages;
      packages.niv = pkgs.niv;
    };
  };
}
