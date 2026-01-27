# Get a locked flake input value that can be given to `builtins.fetchTree` or to
# `builtins.getFlake`. For example,
#
#     pkgs = import (builtins.fetchTree (flakeInput "nixpkgs")) { }
#
# This function can also be used to get inputs of inputs using dot-separated
# paths. For example,
#
#     pkgs = import (builtins.fetchTree (flakeInput "crate2nix_stable.nixpkgs")) { }
#
# Gets the nixpkgs input of the crate2nix_stable input.

let
  flakeLock = builtins.fromJSON (builtins.readFile ../flake.lock);
  flakeInputNodeOf = parentNode: inputName:
    let
      inputNodeName = builtins.getAttr inputName parentNode.inputs;
    in
    builtins.getAttr inputNodeName flakeLock.nodes;
  rootNode = let rootName = flakeLock.root; in builtins.getAttr rootName flakeLock.nodes;
in

name:
let
  parts = builtins.split "[.]" name;
  inputNames = builtins.filter builtins.isString parts;
  flakeNode = builtins.foldl' flakeInputNodeOf rootNode inputNames;
in
flakeNode.locked
