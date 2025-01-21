let
  flakeLock = builtins.fromJSON (builtins.readFile ../flake.lock);
  flakeInputNodeOf = parentNode: inputName:
    let
      inputNodeName = builtins.getAttr inputName parentNode.inputs;
    in
    builtins.getAttr inputNodeName flakeLock.nodes;
  rootNode = let rootName = flakeLock.root; in builtins.getAttr rootName flakeLock.nodes;
in
{
  # Get a locked flake input value that can be given to `builtins.fetchTree` or
  # to `builtins.getFlake`. For example,
  #
  #     pkgs = import (builtins.fetchTree (flakeInput "nixpkgs")) { }
  #
  flakeInput = name: (flakeInputNodeOf rootNode name).locked;

  # Get a locked flake input like `flakeInput`, but instead of taking a single
  # name this function takes a list of nodes to traverse. For example,
  #
  #     flakeNestedInput ["a" "b" "c"]
  #
  # Gets the locked input named "c" which is an input to "b" which is in turn an
  # input to "a"
  flakeNestedInput = names: (builtins.foldl' flakeInputNodeOf rootNode names).locked;
}
