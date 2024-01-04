{
  inputs = {
    nixpkgs.url = "nixpkgs/0cbe9f69c234a7700596e943bfae7ef27a31b735";

    flake-compat.url = "https://flakehub.com/f/edolstra/flake-compat/1.tar.gz";

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    devshell = {
      url = "github:numtide/devshell";
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
      # TODO: Get rid of nixpkgs in sources and just use the flake inputs.
      _module.args.pkgs =
        let sources = import ./nix/sources.nix;
        in import sources.nixpkgs { inherit system; };
      formatter = pkgs.nixpkgs-fmt;
      checks = config.packages;
      packages.niv = pkgs.niv;
    };
  };
}
