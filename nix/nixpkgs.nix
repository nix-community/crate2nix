let
  flakeInput = import ./flakeInput.nix;
in
import (builtins.fetchTree (flakeInput "nixpkgs"))
