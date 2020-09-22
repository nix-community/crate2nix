{ stdenv, crate2nix, callPackage, fetchurl, writeText, lib, linkFarm, crates-io-index }:
src:
let
  inherit (builtins) filter match elemAt replaceStrings;
  lock = builtins.fromTOML (builtins.readFile (src + "/Cargo.lock"));
  parseMetadataKey = key:
    let m = match "checksum ([A-Za-z0-9_-]*) ([A-Za-z0-9_.+-]*) \\(([A-Za-z0-9./_+:-]*)\\)" key;
    in
      assert (! isNull m); {
        name = elemAt m 0;
        version = elemAt m 1;
        source = elemAt m 2;
      };
  parsedMetadata = builtins.attrValues
    (builtins.mapAttrs
      (key: checksum: {
        inherit (parseMetadataKey key) name version;
        inherit checksum;
      }) lock.metadata or { });
  prefetchCrate = crate: fetchurl rec {
    # We use .tar.gz here instead of .crate because this allows Nix to reuse this source for the actual build
    name = "${crate.name}-${crate.version}.tar.gz";
    url = "https://static.crates.io/crates/${crate.name}/${crate.name}-${crate.version}.crate";
    sha256 = crate.checksum;
  };
  tarballToCrate = crate: { path = crate; name = replaceStrings [ ".tar.gz" ] [ ".crate" ] crate.name; };
  crates = map prefetchCrate (filter fetchable lock.package) ++ map prefetchCrate parsedMetadata;
  fetchable = crate: crate ? checksum;
  local-registry = linkFarm "cargo-local-registry"
    ([{ path = crates-io-index; name = "index"; }]
      ++ map tarballToCrate crates);
  cargo-config = writeText "config.toml" ''
    [source.crates-io]
    replace-with = "nix-crates-registry"

    [source.nix-crates-registry]
    local-registry = "${local-registry}"

    [net]
    offline = true
  '';
  cargo-nix = stdenv.mkDerivation {
    name = "Cargo.nix";
    CARGO_HOME = "/tmp/cargo-home";
    inherit src;
    phases = [ "unpackPhase" "buildPhase" ];
    buildInputs = [ crate2nix ];
    buildPhase = ''
      mkdir $CARGO_HOME
      cp ${cargo-config} $CARGO_HOME/config.toml
      crate2nix generate
      cp -r . $out
    '';
  };
in
callPackage "${cargo-nix}/Cargo.nix"
