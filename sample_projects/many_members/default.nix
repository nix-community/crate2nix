# Generates a synthetic Cargo workspace with many members that share most of
# their dependency graph, for exercising `allWorkspaceMembers` /
# `internal.builtRustCratesForRoots`.
#
# * `shared_0` .. `shared_<sharedCrates - 1>`: a chain of libraries, each
#   depending on its predecessor. `shared_0` has a build script with a build
#   dependency and uses a proc-macro.
# * `member_0` .. `member_<members - 1>`: libraries depending on the last
#   `shared_*` crate, so each pulls in the whole chain. Every odd member
#   enables the `extra` feature of `featured`, so that crate (and everything
#   depending on it) comes in two feature variants.
#
# Only path dependencies are used, so no network access is needed.
{ lib
, runCommand
, cargo
, members ? 60
, sharedCrates ? 40
}:
let
  sharedName = i: "shared_${toString i}";
  memberName = i: "member_${toString i}";
  lastShared = sharedName (sharedCrates - 1);

  writeCrate =
    { name
    , dependencies ? ""
    , extra ? ""
    , src ? ""
    }: ''
      mkdir -p ${name}/src
      cat > ${name}/Cargo.toml <<'TOML'
      [package]
      name = "${name}"
      version = "0.1.0"
      edition = "2021"
      ${extra}
      [dependencies]
      ${dependencies}
      TOML
      cat > ${name}/src/lib.rs <<'RS'
      ${src}
      RS
    '';

  crates =
    [
      (writeCrate {
        name = "featured";
        extra = ''
          [features]
          extra = []
        '';
      })
      (writeCrate {
        name = "build_helper";
      })
      (writeCrate {
        name = "macros";
        extra = ''
          [lib]
          proc-macro = true
        '';
      })
      (writeCrate {
        name = sharedName 0;
        dependencies = ''
          featured = { path = "../featured" }
          macros = { path = "../macros" }
          [build-dependencies]
          build_helper = { path = "../build_helper" }
        '';
      })
      "echo 'fn main() {}' > ${sharedName 0}/build.rs"
    ]
    ++ map
      (i: writeCrate {
        name = sharedName i;
        dependencies = ''
          ${sharedName (i - 1)} = { path = "../${sharedName (i - 1)}" }
        '';
      })
      (lib.range 1 (sharedCrates - 1))
    ++ map
      (i: writeCrate {
        name = memberName i;
        dependencies = ''
          ${lastShared} = { path = "../${lastShared}" }
        '' + lib.optionalString (lib.mod i 2 == 1) ''
          featured = { path = "../featured", features = [ "extra" ] }
        '';
      })
      (lib.range 0 (members - 1));
in
assert sharedCrates >= 1;
runCommand "many-members-workspace" { nativeBuildInputs = [ cargo ]; } ''
  mkdir -p $out
  cd $out
  cat > Cargo.toml <<'TOML'
  [workspace]
  resolver = "2"
  members = [ "featured", "build_helper", "macros", "shared_*", "member_*" ]
  TOML
  ${lib.concatStringsSep "\n" crates}
  export CARGO_HOME=$TMPDIR/cargo
  cargo generate-lockfile --offline
''
