{ sources ? import ./nix/sources.nix
, pkgs ? import sources.nixpkgs { config = { }; }
, release ? true
}: pkgs.callPackage ./crate2nix.nix { inherit release; }
