{lib, crate2nix}:

let buildRustCrateFake = args: {features}: args // {inherit features;};
    fakeCrates = {
      "pkg_id1" = buildRustCrateFake {
        crateName = "id1";
      };
      "pkg_id2" = buildRustCrateFake {
        crateName = "id2";
      };
      "pkg_id3" = buildRustCrateFake {
        crateName = "id3";
      };
    };
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
    dependencyDerivations = crates: features: dependencies:
      builtins.map
        (v: lib.getAttrs ["crateName" "features"] v)
        (crate2nix.dependencyDerivations crates features dependencies);
in lib.runTests {

  testForDefaultAndIgnored = {
    expr = dependencyDerivations fakeCrates ["default" "ignored" ] fakeDependencies;
    expected = [
      { "crateName" = "id1"; "features" = ["default"]; }
      { "crateName" = "id3"; "features" = []; }
    ];
  };

  testWithOptional = {
    expr = dependencyDerivations fakeCrates ["default" "optional_id2" ] fakeDependencies;
    expected = [
      { "crateName" = "id1"; "features" = ["default"]; }
      { "crateName" = "id2"; "features" = ["default"]; }
      { "crateName" = "id3"; "features" = []; }
    ];
  };

  testWithDepFeatures = {
    expr = dependencyDerivations
      fakeCrates
        ["default" "id1/default" "id1/stuff" "id2/ignored_feature" "id3/feature1" ]
        fakeDependencies;
    expected = [
      { "crateName" = "id1"; "features" = ["default" "default" "stuff"]; }
      { "crateName" = "id3"; "features" = ["feature1" ]; }
    ];
  };

}
