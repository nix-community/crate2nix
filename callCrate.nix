{ stdenv, crate2nix, callPackage, fetchurl, writeText, lib, linkFarm, crates-io-index }:
{ src
, cargoLock ? src + "/Cargo.lock"
, overrides ? { } }:
let
  inherit (builtins) filter match elemAt replaceStrings;

  lock = builtins.fromTOML (builtins.readFile cargoLock);

  parseMetadataKey = key:
  let m = match "checksum ([A-Za-z0-9_-]*) ([A-Za-z0-9_.+-]*) \\(([A-Za-z0-9./_+:-]*)\\)" key;
  in assert (! isNull m); {
    name = elemAt m 0;
    version = elemAt m 1;
    source = elemAt m 2;
  };

  parsedMetadata = builtins.attrValues (builtins.mapAttrs (key: checksum: {
    inherit (parseMetadataKey key) name version;
    inherit checksum;
  }) lock.metadata or {});

  prefetch-crate = crate: fetchurl rec {
    # We use .tar.gz here instead of .crate because this allows Nix to reuse this source for the actual build
    name = "${crate.name}-${crate.version}.tar.gz";
    url = "https://static.crates.io/crates/${crate.name}/${crate.name}-${crate.version}.crate";
    sha256 = crate.checksum;
  };

  crates = map prefetch-crate (filter fetchable lock.package) ++ map prefetch-crate parsedMetadata;

  fetchable = crate: crate ? checksum;

  source-repository = linkFarm "cargo-source-repository"
    ([ { path = crates-io-index; name = "index"; } ]
    ++ map (crate: { path = crate; name = replaceStrings [".tar.gz"] [".crate"] crate.name; }) crates);

  cargoConfig = writeText "config.toml" ''
    [source.crates-io]
    replace-with = "nix-crates-registry"

    [source.nix-crates-registry]
    local-registry = "${source-repository}"
  '';

  cargo-nix = stdenv.mkDerivation {
    name = "Cargo.nix";
    CARGO_HOME = "/tmp/cargo-home";
    inherit src;
    phases = [ "unpackPhase" "buildPhase" ];
    buildInputs = [ crate2nix ];
    buildPhase = ''
      mkdir $CARGO_HOME
      cp ${cargoConfig} $CARGO_HOME/config.toml
      crate2nix generate
      cp -r . $out
    '';
  };
in callPackage "${cargo-nix}/Cargo.nix" overrides
