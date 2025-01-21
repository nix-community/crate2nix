let
  flakeInput = (import ./lib.nix).flakeInput;
in
import (builtins.fetchTree (flakeInput "nixpkgs"))
