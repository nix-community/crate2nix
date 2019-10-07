#
# Some tools that might be useful in builds.
#
# Part of the "public" API of crate2nix in the sense that we will try to
# avoid breaking the API and/or mention breakages in the CHANGELOG.
#

{ pkgs? import ./nixpkgs.nix { config = {}; }}:

let cargo_nix = pkgs.callPackage ./Cargo.nix {};
    crate2nix = cargo_nix.rootCrate.build;
    generate = {name, src, cargoToml}: pkgs.stdenv.mkDerivation ({
      name = "${name}-crate2nix";

      buildInputs = [ pkgs.cargo crate2nix ];

      buildCommand = ''
          mkdir -p "$out/cargo"

          export CARGO_HOME="$out/cargo"
          export HOME="$out"

          # If we need to write the lock file, we make
          # a copy.
          cargo metadata -q --locked >/dev/null || {
            echo Copying sources to make Cargo.lock writeable.
            cp -apR "${src}" "$out/src"
            src="$out/src"
          }

          crate2nix generate \
           -f ${src}/${cargoToml} \
           -o $out/default.nix
      '';
   });
in {
  #
  # Returns a derivation for a rust binary package.
  #
  # name: will be part of the derivation name
  # src: the source that is needed to build the crate, usually the crate/workspace root directory
  # cargoToml: Path to the Cargo.toml file relative to src, "Cargo.toml" by default.
  #
  generated = {name, src, cargoToml? "Cargo.toml"}:
    pkgs.callPackage (generate {inherit name src cargoToml;}) {};
}

