{lib, crate2nix}:

let buildRustCrateFake = lib.id;
    fakeDependencies = {
      "id1" = "pkg_id1";
      "optional_id2" = {
        package_id = "pkg_id2";
        optional = true;
      };
      "id3" = {
        package_id = "pkg_id3";
        uses_default_features = false;
      };
    };
    dependencyDerivations = features: dependencies:
      crate2nix.dependencyDerivations buildRustCrateFake features dependencies;
in lib.runTests {

  testForDefaultAndIgnored = {
    expr = dependencyDerivations ["default" "ignored" ] fakeDependencies;
    expected = [
      "pkg_id1"
      "pkg_id3"
    ];
  };

  testWithOptional = {
    expr = dependencyDerivations ["default" "optional_id2" ] fakeDependencies;
    expected = [
      "pkg_id1"
      "pkg_id3"
      "pkg_id2"
    ];
  };

  testWithDepFeatures = {
    expr = dependencyDerivations
        ["default" "id1/default" "id1/stuff" "id2/ignored_feature" "id3/feature1" ]
        fakeDependencies;
    expected = [
      "pkg_id1"
      "pkg_id3"
    ];
  };

}
