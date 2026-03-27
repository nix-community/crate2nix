# Shared wrapper logic for the crate2nix binary.
# Takes the raw rootCrate derivation and wraps it with PATH, completions, etc.
{ pkgs ? (
    import (builtins.fetchTree (import ../nix/flakeInput.nix "nixpkgs")) { }
  )
, stdenv ? pkgs.stdenv
, lib ? pkgs.lib
, symlinkJoin ? pkgs.symlinkJoin
, makeWrapper ? pkgs.makeWrapper
, nix ? pkgs.nix
, cargo ? pkgs.cargo
, libsecret ? pkgs.libsecret
, nix-prefetch-git ? pkgs.nix-prefetch-git
, release ? true
  # The raw rootCrate.build derivation to wrap.
, rootCrate
}:
let
  crate2nix = rootCrate.overrideAttrs (attrs: {
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
  passthru = {
    crate = crate2nix;
  };
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
      lib.maintainers.domenkozar
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
