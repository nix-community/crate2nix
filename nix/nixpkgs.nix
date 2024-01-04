let
  flakeLock = builtins.fromJSON (builtins.readFile ../flake.lock);
in
import "${builtins.fetchTree flakeLock.nodes.nixpkgs.locked}"
