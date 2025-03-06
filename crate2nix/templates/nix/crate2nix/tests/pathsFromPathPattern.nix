{ lib, ... }:
let
  module = import ../../../../../nix/lib/paths-from-path-pattern.nix { inherit lib; };
  pathToRegex = module.pathToRegex;
  pathsFromPathPattern = module.pathsFromPathPattern;
in
{
  testPathsFromPathPattern = {
    expr = pathsFromPathPattern ./. "p*sFro[l-p]PathP?tter[^x].{nix,xin}";
    expected = [ "pathsFromPathPattern.nix" ];
  };
  testGlobStar = {
    expr = [
      (pathToRegex "crates/*")
      (builtins.match (pathToRegex "crates/*") "crates/main" != null)
    ];
    expected = [ "crates[/][^/]*" true ];
  };
  testGlobDoubleStar = {
    expr = [
      (pathToRegex "crates/**/*")
      (builtins.match (pathToRegex "crates/**/*") "crates/prod/main" != null)
    ];
    expected = [ "crates[/].*[/][^/]*" true ];
  };
  testGlobQuestionMark = {
    expr = [
      (pathToRegex "crate?/*")
      (builtins.match (pathToRegex "crate?/*") "crates/main" != null)
    ];
    expected = [ "crate[^/][/][^/]*" true ];
  };
  testGlobAlternative = {
    expr = [
      (pathToRegex "Cargo.{toml,lock}")
      (builtins.match (pathToRegex "Cargo.{toml,lock}") "Cargo.toml" != null)
    ];
    expected = [ "Cargo[.](toml|lock)" true ];
  };
  testGlobRange = {
    expr = [
      (pathToRegex "artifacts/v[0-9].tgz")
      (builtins.match (pathToRegex "artifacts/v[0-9].tgz") "artifacts/v3.tgz" != null)
    ];
    expected = [ "artifacts[/]v[0-9][.]tgz" true ];
  };
  testGlobCaretNegation = {
    expr = [
      (pathToRegex "h[^w-x]")
      (builtins.match (pathToRegex "h[^w-x]") "hi" != null)
      (builtins.match (pathToRegex "h[^w-x]") "hx" != null)
    ];
    expected = [ "h[^w-x]" true false ];
  };
}
