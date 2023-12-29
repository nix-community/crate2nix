{
  description = "crate2nix generates nix build files for rust crates using cargo";

  inputs = {  

  };

  outputs = inputs@{ nixpkgs, self, flake-parts }: flake-parts.lib.mkFlake { inherit inputs; } {
    systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];

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
