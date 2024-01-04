{
  flake.overlays.default = final: prev: {
    crate2nix = prev.callPackage ./default.nix { };
  };

  perSystem =
    { pkgs
    , ...
    }: {
      packages.default = pkgs.callPackage ./default.nix { };
    };
}
