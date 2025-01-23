let
  flakeInput = import ./flakeInput.nix;
  src = builtins.fetchTree (flakeInput "nix-test-runner");

  # Use last pinned crate2nix packages and corresponding nixpkgs to build the
  # test runner so that it works even if we have broken stuff!
  crate2nix_stable = builtins.fetchTree (flakeInput "crate2nix_stable");

  # For consistency we should use the locked version of nixpkgs from the
  # crate2nix_stable release which would be,
  #
  #     nixpkgs_stable = builtins.fetchTree (flakeInput "crate2nix_stable.nixpkgs");
  #
  # But nix-test-runner fails to build with the crate2nix_stable and the pinned
  # nixpkgs from the same release. That's because "crate2nix_stable.nixpkgs"
  # provides cargo v1.78 while "nixpkgs" provides cargo v1.78. Between those two
  # cargo versions `cargo metadata` changed the format it uses for package ID
  # strings which breaks checksum lookups in legacy V1 Cargo.toml files in 
  # crate2nix - and it happens that nix-test-runner uses a V1 Cargo.toml
  # manifest.
  #
  # So for the time being it is necessary to use the current "nixpkgs" here
  # which ended up on an older revision with an older cargo until there is
  # a stable release of crate2nix that fixes V1 manifest parsing with the newer
  # cargo, at which time this whole note may be removed.
  nixpkgs_stable = builtins.fetchTree (flakeInput "nixpkgs");
in
{
  # A system must be specified if using default value for pkgs and calling this
  # package from a pure evaluation context, such as from the flake devShell.
  system ? builtins.currentSystem
, pkgs ? import nixpkgs_stable { inherit system; }
, tools ? pkgs.callPackage "${crate2nix_stable}/tools.nix" { }
}:
let
  nixTestRunner = tools.appliedCargoNix {
    name = "nix-test-runner";
    inherit src;
  };
in
nixTestRunner.rootCrate.build
