let
  tests = import ../nix/nixpkgs.nix { config = { }; };
in
{
  x86_64-linux = tests;
  # Uncomment to test build on macOS too
  # x86_64-darwin = {};
}
