{ inputs, ... }:
let
  overlays = [
    (import inputs.rust-overlay)
    (self: super: assert !(super ? rust-toolchain); rec {
      rust-toolchain = super.rust-bin.fromRustupToolchainFile ../../rust-toolchain.toml;

      # buildRustCrate/crate2nix depend on this.
      rustc = rust-toolchain;
      cargo = rust-toolchain;
    })
  ];
in
{
  perSystem = { system, ... }: {
    _module.args.pkgs = import inputs.nixpkgs { inherit system overlays; config = { }; };
  };
}
