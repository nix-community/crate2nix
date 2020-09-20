{ sources ? import ./nix/sources.nix
, pkgs ? import sources.nixpkgs { config = { }; }
, release ? true
, crates-io-index ? builtins.trace
  "You may wish to pin crates-io-index to a version of https://github.com/rust-lang/crates.io-index"
  (builtins.fetchTarball https://github.com/rust-lang/crates.io-index/archive/master.tar.gz)
}:
let
  crate2nix = pkgs.callPackage ./crate2nix.nix { inherit release; };
  callCrate = pkgs.callPackage ./callCrate.nix { inherit crate2nix crates-io-index; };
in crate2nix // { inherit callCrate; }
