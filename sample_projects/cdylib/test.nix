{ pkgs ? import ../../nix/nixpkgs.nix { config = {}; }, generatedCargoNix }:
let
  basePackage = pkgs.callPackage generatedCargoNix {};
  lib = basePackage.rootCrate.build.lib;
  src = pkgs.writeText "main.c" ''
    extern void some_function(void);

    int main(int argc, char** argv) {
       some_function();
    }
  '';
in
pkgs.runCommandCC "link-cdylib" {
  crateName = "cdylib";
} ''
  mkdir -p $out/bin
  cc -L${lib}/lib/ -lcdylib ${src} -o $out/bin/cdylib
''
