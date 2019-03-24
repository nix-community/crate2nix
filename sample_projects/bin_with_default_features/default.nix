
{ pkgs? import <nixos-unstable> { config = {}; },
  callPackage? pkgs.callPackage,
  stdenv? pkgs.stdenv,
  buildRustCrate? pkgs.buildRustCrate,
  fetchurl? pkgs.fetchurl }:

rec {
    root_crate = crates."bin_with_default_features 0.1.0 (path+file:///home/peter/gdrive/projects/cargo2nix/sample_projects/bin_with_default_features)";
    crates = {
        "bin_with_default_features 0.1.0 (path+file:///home/peter/gdrive/projects/cargo2nix/sample_projects/bin_with_default_features)"
            = buildRustCrate {
                crateName = "bin_with_default_features";
                version = "0.1.0";
                src = /home/peter/gdrive/projects/cargo2nix/sample_projects/bin_with_default_features;
                edition = "2018";
                authors = [
                    "Peter Kolloch <info@eigenvalue.net>"
                ];
                dependencies = [
                    crates."hello_world_lib 0.1.0 (path+file:///home/peter/gdrive/projects/cargo2nix/sample_projects/lib)"
                ];
                features = [
                   "default"
                   "hello_world_lib"
                ];
            };
        "hello_world_lib 0.1.0 (path+file:///home/peter/gdrive/projects/cargo2nix/sample_projects/lib)"
            = buildRustCrate {
                crateName = "hello_world_lib";
                version = "0.1.0";
                src = /home/peter/gdrive/projects/cargo2nix/sample_projects/lib;
                edition = "2018";
                libPath = "src/lib.rs";
                authors = [
                    "Peter Kolloch <info@eigenvalue.net>"
                ];
            };
    };
}

