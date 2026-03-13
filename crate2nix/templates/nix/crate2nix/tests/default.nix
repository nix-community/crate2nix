let
  pkgs = import ../../../../../nix/nixpkgs.nix { };
in
pkgs.callPackage ./tests.nix { }
