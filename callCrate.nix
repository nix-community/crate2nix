{ stdenv, crate2nix, callPackage, fetchurl, writeText, lib, runCommand, crates-io-index }:
{ src
, cargoLock ? src + "/Cargo.lock"
, overrides ? { } }:
let
  inherit (builtins) filter match elemAt;

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

  prefetch-crate = crate: fetchurl {
    url = "https://static.crates.io/crates/${crate.name}/${crate.name}-${crate.version}.crate";
    sha256 = crate.checksum;
  };

  crates = map prefetch-crate (filter fetchable lock.package) ++ map prefetch-crate parsedMetadata;

  fetchable = crate: crate ? checksum;

  source-repository = runCommand "cargo-source-repository" { }
  (''
    mkdir $out
    cp ${crates-io-index} -r $out/index
  '' + lib.concatMapStringsSep "\n" (crate: "cp ${crate} $out/${crate.name}") crates);

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
