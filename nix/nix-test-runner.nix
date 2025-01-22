let
  flakeInput = (import ./lib.nix).flakeInput;
  src = builtins.fetchTree (flakeInput "nix-test-runner");

  # Use last pinned crate2nix packages and corresponding nixpkgs to build the
  # test runner so that it works even if we have broken stuff!
  crate2nix_stable = builtins.fetchTree (flakeInput "crate2nix_stable");

  # The latest stable crate2nix doesn't work with the version of nixpkgs that is
  # currently locked because of a change to the way `cargo metadata` formats
  # package IDs. So to build the test runner with the last release of crate2nix
  # we also need an older version of nixpkgs. For consistency with using the
  # pinned version of crate2nix we should also use the pinned version of nixpkgs
  # from the same release which would look like this:
  #
  #     builtins.fetchTree (flakeInput "crate2nix_stable.nixpkgs")
  #
  # Unfortunately that doesn't work either. That's likely because ./nixpkgs.nix
  # has been erroneously using the pinned version of nixpkgs from this repo's
  # cachix dependency instead of the version of nixpkgs pinned in this repo's
  # flake. Because that has been fixed in ./nixpkgs.nix it's neccessary to
  # override the pinned stable nixpkgs revision here until the next stable
  # release when it can be matched with the pinned stable crate2nix version
  # again.
  known_good_nixpkgs = builtins.fetchTree {
    "lastModified" = 1700612854;
    "narHash" = "sha256-yrQ8osMD+vDLGFX7pcwsY/Qr5PUd6OmDMYJZzZi0+zc=";
    "owner" = "NixOS";
    "repo" = "nixpkgs";
    "rev" = "19cbff58383a4ae384dea4d1d0c823d72b49d614";
    "type" = "github";
  };
in
{
  # A system must be specified if using default value for pkgs and calling this
  # package from a pure evaluation context, such as from the flake devShell.
  system ? builtins.currentSystem
, pkgs ? import known_good_nixpkgs { inherit system; }
, tools ? pkgs.callPackage "${crate2nix_stable}/tools.nix" { }
}:
let
  nixTestRunner = tools.appliedCargoNix {
    name = "nix-test-runner";
    inherit src;
  };
in
nixTestRunner.rootCrate.build
