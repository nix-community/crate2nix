let
  flakeInput = import ../flakeInput.nix;
  src = builtins.fetchTree (flakeInput "nix-test-runner");
in
{ pkgs
, crate2nixTools
}:
let
  nixTestRunner = crate2nixTools.appliedCargoNix {
    name = "nix-test-runner";
    inherit src;
  };
in
nixTestRunner.rootCrate.build
