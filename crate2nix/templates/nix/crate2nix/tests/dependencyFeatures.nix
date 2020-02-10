{ lib, crate2nix }:

let
  dependencies = [
    {
      name = "tls";
      packageId = "pkgid_tls";
    }
    {
      name = "extra";
      packageId = "pkgid_extra";
    }
    {
      name = "with_target";
      target_cfg = true;
      optional = false;
      with_defaults = false;
      packageId = "pkgid_target";
    }
  ];
in
lib.runTests {

  testStringDependency = {
    expr = crate2nix.dependencyFeatures [] { name = "my_dep"; packageId = "pkg_id"; };
    expected = [ "default" ];
  };

  testWithDefaultUnsetDependency = {
    expr = crate2nix.dependencyFeatures
      []
      { name = "my_dep"; };
    expected = [ "default" ];
  };

  testWithDefaultDependency = {
    expr = crate2nix.dependencyFeatures
      []
      { name = "my_dep"; usesDefaultFeatures = true; };
    expected = [ "default" ];
  };

  testWithDefaultDisabledDependency = {
    expr = crate2nix.dependencyFeatures
      []
      { name = "my_dep"; usesDefaultFeatures = false; };
    expected = [];
  };


  testDependencyFeature = {
    expr = crate2nix.dependencyFeatures [ "my_dep/feature1" ] { name = "my_dep"; packageId = "pkg_id"; };
    expected = [ "default" "feature1" ];
  };

  testDependencyFeatures = {
    expr = crate2nix.dependencyFeatures
      [ "irrelevant" "my_dep/feature1" "my_dep/feature2" "my_dep3/irrelevant2" ]
      { name = "my_dep"; packageId = "pkg_id"; };
    expected = [ "default" "feature1" "feature2" ];
  };

  testDependencyFeatures2 = {
    expr = crate2nix.dependencyFeatures
      [ "irrelevant" "my_dep/feature1" "my_dep/feature2" "my_dep3/irrelevant2" ]
      { name = "my_dep"; usesDefaultFeatures = true; };
    expected = [ "default" "feature1" "feature2" ];
  };

  testDependencyFeaturesWithoutDefault = {
    expr = crate2nix.dependencyFeatures
      [ "irrelevant" "my_dep/feature1" "my_dep/feature2" "my_dep3/irrelevant2" ]
      { name = "my_dep"; usesDefaultFeatures = false; };
    expected = [ "feature1" "feature2" ];
  };
}
