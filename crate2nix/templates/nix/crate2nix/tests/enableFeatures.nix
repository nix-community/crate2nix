{ lib, crate2nix }:
let
  exampleDependency = {
    name = "dep";
    packageId = "pkgid_dep";
    optional = true;
  };
  renamedDependency = {
    name = "dep";
    rename = "dep2";
    packageId = "pkgid_dep2";
    optional = true;
  };
in
{
  testWithNoEnabledDependencies = {
    expr = crate2nix.enableFeatures
      [ exampleDependency ]
      [ "default" ];
    expected = [ "default" ];
  };

  testWithDirectlyEnabledDependency = {
    expr = crate2nix.enableFeatures
      [ exampleDependency ]
      [ "default" "dep" ];
    expected = [ "default" "dep" ];
  };

  testWithDirectlyEnabledRenamedDependency = {
    expr = crate2nix.enableFeatures
      [ renamedDependency ]
      [ "default" "dep2" ];
    expected = [ "default" "dep2" ];
  };

  testWithIndirectlyEnabledDependency = {
    expr = crate2nix.enableFeatures
      [ exampleDependency ]
      [ "default" "dep/feat" ];
    expected = [ "default" "dep" "dep/feat" ];
  };

  testWithIndirectlyEnabledRenamedDependency = {
    expr = crate2nix.enableFeatures
      [ renamedDependency ]
      [ "default" "dep2/feat" ];
    expected = [ "default" "dep2" "dep2/feat" ];
  };

  testWithDuplicateDependencies = {
    expr = crate2nix.enableFeatures
      [ exampleDependency renamedDependency ]
      [ "default" "dep/feat" "dep2/feat" ];
    expected = [ "default" "dep" "dep/feat" "dep2" "dep2/feat" ];
  };

  testWithDisabledRenamedDependency = {
    expr = crate2nix.enableFeatures
      [ renamedDependency ]
      [ "default" "dep/feat" ];
    expected = [ "default" "dep/feat" ];
  };
}
