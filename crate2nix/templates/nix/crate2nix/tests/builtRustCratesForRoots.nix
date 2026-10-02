{ lib, crate2nix, stdenv }:
let
  crate = name: attrs: {
    crateName = name;
    version = "1.0.0";
    edition = "2021";
    src = "/fake/src/${name}";
  } // attrs;
  crateConfigs = {
    "pkg_root_a" = crate "root_a" {
      dependencies = [
        { name = "common"; packageId = "pkg_common"; }
        { name = "lib_b"; packageId = "pkg_lib_b"; rename = "b_renamed"; }
      ];
      devDependencies = [
        { name = "dev_dep"; packageId = "pkg_dev_dep"; }
      ];
    };
    "pkg_root_b" = crate "root_b" {
      dependencies = [
        { name = "common"; packageId = "pkg_common"; features = [ "extra" ]; }
        { name = "macros"; packageId = "pkg_macros"; }
      ];
      buildDependencies = [
        { name = "helper"; packageId = "pkg_helper"; }
      ];
    };
    "pkg_root_c" = crate "root_c" {
      dependencies = [
        { name = "common"; packageId = "pkg_common"; }
      ];
    };
    "pkg_common" = crate "common" {
      dependencies = [
        { name = "leaf"; packageId = "pkg_leaf"; }
      ];
      features = {
        "default" = [ ];
        "extra" = [ ];
      };
    };
    "pkg_leaf" = crate "leaf" { };
    "pkg_lib_b" = crate "lib_b" { };
    "pkg_dev_dep" = crate "dev_dep" { };
    "pkg_helper" = crate "helper" {
      dependencies = [
        { name = "leaf"; packageId = "pkg_leaf"; }
      ];
    };
    "pkg_macros" = crate "macros" { procMacro = true; };
  };
  roots = [ "pkg_root_a" "pkg_root_b" "pkg_root_c" ];
  # Returns the config passed to `buildRustCrate` instead of a derivation, so
  # whole dependency trees can be compared.
  buildRustCrateForPkgsFunc = _: crateConfig: crateConfig;
  perRoot =
    runTests: packageId:
    (crate2nix.builtRustCratesWithFeatures {
      inherit packageId crateConfigs buildRustCrateForPkgsFunc runTests;
      features = [ "default" ];
    }).crates.${packageId};
  shared =
    runTests:
    crate2nix.builtRustCratesForRoots {
      inherit crateConfigs buildRustCrateForPkgsFunc runTests;
      packageIds = roots;
      features = [ "default" ];
    };
  perRootAll = runTests: lib.genAttrs roots (perRoot runTests);
  commonFeatures =
    root: map (d: d.features) (lib.filter (d: d.crateName == "common") root.dependencies);
in
{
  testSameAsPerRoot = {
    expr = shared false;
    expected = perRootAll false;
  };

  testSameAsPerRootWithTests = {
    expr = shared true;
    expected = perRootAll true;
  };

  testDevDependenciesOnlyWithTests = {
    expr = map (d: d.crateName) (shared true).pkg_root_a.dependencies;
    expected = [ "common" "lib_b" "dev_dep" ];
  };

  # `common` is reached with different features from different roots and
  # must not be shared between them.
  testFeatureVariantsStayDistinct = {
    expr = lib.mapAttrs (_: commonFeatures) (shared false);
    expected = {
      pkg_root_a = [ [ "default" ] ];
      pkg_root_b = [ [ "default" "extra" ] ];
      pkg_root_c = [ [ "default" ] ];
    };
  };

  testSingleRoot = {
    expr = crate2nix.builtRustCratesForRoots {
      inherit crateConfigs buildRustCrateForPkgsFunc;
      packageIds = [ "pkg_root_b" ];
      features = [ "default" ];
      runTests = false;
    };
    expected = { pkg_root_b = perRoot false "pkg_root_b"; };
  };
}
