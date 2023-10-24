{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, flake-utils }: {
    overlays.default = final: prev: {
      crate2nix = self.callPackage ./default.nix { };
    };
    templates.default = {
      path = ./template;
      description = "An example of crate2nix";
    };
  } // flake-utils.lib.eachDefaultSystem (system:
    let
      sources = import ./nix/sources.nix;
      pkgs = import sources.nixpkgs { inherit system; };
    in
    {
      formatter = pkgs.nixpkgs-fmt;
      packages = rec {
        crate2nix = pkgs.callPackage ./default.nix { };
        default = crate2nix;
      };
    });
}
