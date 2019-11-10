{ pkgs? import ./nixpkgs.nix { config = {}; },
  lib? pkgs.lib,
  cargo? pkgs.cargo,
  nix? pkgs.nix,
  makeWrapper? pkgs.makeWrapper,
  callPackage? pkgs.callPackage}:

let cargo_nix = callPackage ./crate2nix/Cargo.nix {};
    crate2nix = cargo_nix.rootCrate.build;

in pkgs.symlinkJoin {
  name = crate2nix.name;
  paths = [ crate2nix ];
  buildInputs = [ makeWrapper cargo ];
  postBuild = ''
    # Fallback to built dependencies for cargo and nix-prefetch-url
    wrapProgram $out/bin/crate2nix \
       --suffix PATH ":" ${lib.makeBinPath [ cargo nix ]}
    rm -rf $out/lib $out/bin/crate2nix.d
    mkdir -p \
      $out/share/bash-completion/completions \
      $out/share/zsh/vendor-completions
    $out/bin/crate2nix completions -s 'bash' -o $out/share/bash-completion/completions
    $out/bin/crate2nix completions -s 'zsh' -o $out/share/zsh/vendor-completions
  '';
}
