# Provided by callPackage or also directly usable via nix-build with defaults.
{ pkgs ? (
    let
      flakeLock = builtins.fromJSON (builtins.readFile ../flake.lock);
    in
    import "${builtins.fetchTree flakeLock.nodes.nixpkgs.locked}" { }
  )
, stdenv ? pkgs.stdenv
, lib ? pkgs.lib
, symlinkJoin ? pkgs.symlinkJoin
, makeWrapper ? pkgs.makeWrapper
, darwin ? pkgs.darwin
, defaultCrateOverrides ? pkgs.defaultCrateOverrides
, nix ? pkgs.nix
, cargo ? pkgs.cargo
, libsecret ? pkgs.libsecret
, callPackage ? pkgs.callPackage
, nix-prefetch-git ? pkgs.nix-prefetch-git
  # Seperate arguements that are NOT filled by callPackage.
, cargoNixPath ? ./Cargo.nix
, release ? true
}:
let
  cargoNix = callPackage cargoNixPath { inherit release; };
  withoutTemplates = name: type:
    let
      baseName = builtins.baseNameOf (builtins.toString name);
    in
      !(baseName == "templates" && type == "directory");
  crate2nix = (cargoNix.rootCrate.build.override {
    testCrateFlags = [
      "--skip nix_integration_tests"
    ];
    crateOverrides = defaultCrateOverrides // {
      crate2nix = { src, ... }: {
        src =
          if release
          then src
          else
            lib.cleanSourceWith {
              filter = withoutTemplates;
              inherit src;
            };
        dontFixup = !release;
      };
      cssparser-macros = attrs: assert builtins.trace "cssparser" true;{
        buildInputs = lib.optionals stdenv.isDarwin [ darwin.apple_sdk.frameworks.Security ];
      };
    };
  }).overrideAttrs (attrs: {
    postInstall = lib.optionalString stdenv.isLinux ''
      patchelf --add-needed ${libsecret}/lib/libsecret-1.so.0 $out/bin/crate2nix
    '';
  });
  set_templates = if release then "" else "--set TEMPLATES_DIR ${./templates}";
in
symlinkJoin {
  name = crate2nix.name;
  paths = [ crate2nix ];
  buildInputs = [ makeWrapper cargo ];
  meta = {
    description = "Nix build file generator for rust crates.";
    longDescription = ''
      Crate2nix generates nix files from Cargo.toml/lock files
      so that you can build every crate individually in a nix sandbox.
    '';
    homepage = "https://github.com/nix-community/crate2nix";
    license = lib.licenses.asl20;
    maintainers = [
      {
        github = "kolloch";
        githubId = 339354;
        name = "Peter Kolloch";
      }
      lib.maintainers.andir
    ];
    mainProgram = "crate2nix";
    platforms = lib.platforms.all;
  };
  postBuild = ''
    # Fallback to built dependencies for cargo and nix-prefetch-url
    wrapProgram $out/bin/crate2nix ${set_templates}\
      --suffix PATH ":" ${lib.makeBinPath [ cargo nix nix-prefetch-git ]}
    rm -rf $out/lib $out/bin/crate2nix.d
    mkdir -p \
      $out/share/bash-completion/completions \
      $out/share/zsh/vendor-completions
    $out/bin/crate2nix completions -s 'bash' -o $out/share/bash-completion/completions
    $out/bin/crate2nix completions -s 'zsh' -o $out/share/zsh/vendor-completions
  '';
}
