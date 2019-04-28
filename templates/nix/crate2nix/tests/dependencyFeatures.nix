{lib, crate2nix}:

let dependencies = {
      "tls" = "pkgid_tls";
      "extra" = "pkgid_extra";
      "with_target" = {
        target_cfg = true;
        optional = false;
        with_defaults = false;
        package_id = "pkgid_target";
      };
    };
in lib.runTests {

  testStringDependency = {
    expr = crate2nix.dependencyFeatures [] "my_dep" "pkg_id";
    expected = ["default"];
  };

  testWithDefaultUnsetDependency = {
    expr = crate2nix.dependencyFeatures
             []
             "my_dep"
             {};
    expected = ["default"];
  };

  testWithDefaultDependency = {
    expr = crate2nix.dependencyFeatures
             []
             "my_dep"
             { uses_default_features = true; };
    expected = ["default"];
  };

  testWithDefaultDisabledDependency = {
    expr = crate2nix.dependencyFeatures
             []
             "my_dep"
             { uses_default_features = false; };
    expected = [];
  };


  testDependencyFeature = {
    expr = crate2nix.dependencyFeatures ["my_dep/feature1"] "my_dep" "pkg_id";
    expected = [ "default" "feature1" ];
  };

  testDependencyFeatures = {
    expr = crate2nix.dependencyFeatures
      [ "irrelevant" "my_dep/feature1" "my_dep/feature2" "my_dep3/irrelevant2" ]
      "my_dep" "pkg_id";
    expected = [ "default" "feature1" "feature2" ];
  };

  testDependencyFeatures2 = {
    expr = crate2nix.dependencyFeatures
      [ "irrelevant" "my_dep/feature1" "my_dep/feature2" "my_dep3/irrelevant2" ]
      "my_dep" { uses_default_features = true; };
    expected = [ "default" "feature1" "feature2" ];
  };

  testDependencyFeaturesWithoutDefault = {
    expr = crate2nix.dependencyFeatures
      [ "irrelevant" "my_dep/feature1" "my_dep/feature2" "my_dep3/irrelevant2" ]
      "my_dep" { uses_default_features = false; };
    expected = [ "feature1" "feature2" ];
  };
}
