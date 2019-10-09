{ nixpkgs ? null, ... }@_args:
with builtins;
let
  names = filter (n: _args.${n} != null) (attrNames _args);
  args = listToAttrs (map (name: { inherit name; value = _args.${name}; }) names);in
{
  crate2nix = import ./default.nix args;
}
