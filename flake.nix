{
  description = "crate2nix generates nix build files for rust crates using cargo";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.2311.553775.tar.gz";

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

  outputs = inputs@{ nixpkgs, self, flake-parts, devshell, pre-commit-hooks }: flake-parts.lib.mkFlake { inherit inputs; } {
    systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];

    imports = [
      ./nix/devshell/flake-module.nix
      ./nix/pre-commit/flake-module.nix
      ./docs/flake-module.nix
    ];

    flake = {
      overlays.default = final: prev: {
        crate2nix = self.callPackage ./default.nix { };
      };
      templates.default = {
        path = ./template;
        description = "An example of crate2nix";
      };
    };

    perSystem = { config, self', inputs', pkgs, system, ... }: {
      formatter = pkgs.nixpkgs-fmt;
      checks = config.packages;
      packages = rec {
        crate2nix = pkgs.callPackage ./default.nix { };
        default = crate2nix;
      };
    };
  };
}
