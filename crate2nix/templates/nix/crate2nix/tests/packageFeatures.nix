{ lib, crate2nix, stdenv }:
let
  crateConfigs = {
    "pkg_root" = {
      crateName = "id1";
      features = {
        "optional_id2" = [ ];
      };
      dependencies = [
        {
          name = "id1";
          packageId = "pkg_id1";
        }
        {
          name = "optional_id2";
          packageId = "pkg_id2";
          optional = true;
        }
        {
          name = "id3";
          packageId = "pkg_id3";
          usesDefaultFeatures = false;
        }
      ];
    };
    "pkg_with_feature_clash" = {
      dependencies = [
        {
          name = "id1";
          packageId = "pkg_id1";
        }
      ];
      buildDependencies = [
        {
          name = "id1";
          packageId = "pkg_id1";
          features = [ "for_build" ];
        }
      ];
    };
    "pkg_id1" = {
      crateName = "id1";
      features = {
        "default" = [ ];
      };
    };
    "pkg_id2" = {
      crateName = "id2";
      features = { };
    };
    "pkg_id3" = {
      crateName = "id3";
      features = { };
    };

    # A crate that declares no `default` feature, depended on with default
    # features left on (the implicit default for the edge). Cargo enables
    # nothing in this case, so neither should crate2nix.
    "pkg_no_default_root" = {
      crateName = "no_default_root";
      dependencies = [
        {
          name = "id3";
          packageId = "pkg_id3";
        }
      ];
    };

    "pkg_numtest" = {
      crateName = "numtest";
      dependencies = [
        {
          name = "num";
          packageId = "pkg_num";
        }
      ];
    };

    "pkg_num_bigint" = {
      crateName = "num-bigint";
    };

    "pkg_num" = {
      crateName = "num";
      dependencies = [
        {
          name = "num-bigint";
          packageId = "pkg_num_bigint";
          usesDefaultFeatures = false;
          optional = true;
        }
      ];
      features = {
        "default" = [ "std" ];
        "std" = [ "num-bigint/std" ];
      };
    };
  };
  packageFeatures =
    packageId: features:
    crate2nix.mergePackageFeatures
      {
        target = crate2nix.makeDefaultTarget stdenv.hostPlatform;
        runTests = false;
        rootPackageId = packageId;
        inherit crateConfigs packageId features;
      };
in
{

  testNumDependencies = {
    expr = packageFeatures "pkg_num" [ "default" ];
    expected = {
      "pkg_num" = [ "default" "num-bigint" "num-bigint/std" "std" ];
      "pkg_num_bigint" = [ "std" ];
    };
  };

  testNumTestDependencies = {
    expr = packageFeatures "pkg_numtest" [ "default" ];
    expected = {
      # pkg_numtest declares no `default` feature → "default" enables nothing.
      "pkg_numtest" = [ ];
      "pkg_num" = [ "default" "num-bigint" "num-bigint/std" "std" ];
      "pkg_num_bigint" = [ "std" ];
    };
  };

  testTerminalPackageDependency = {
    expr = packageFeatures "pkg_id1" [ ];
    expected = {
      "pkg_id1" = [ ];
    };
  };

  testTerminalPackageDependencyWithDefault = {
    expr = packageFeatures "pkg_id1" [ "default" ];
    expected = {
      "pkg_id1" = [ "default" ];
    };
  };

  testRootPackage = {
    expr = packageFeatures "pkg_root" [ "default" ];
    expected = {
      # pkg_root declares no `default` feature, so the requested "default"
      # enables nothing — matching `cargo build` (previously [ "default" ]).
      "pkg_root" = [ ];
      "pkg_id1" = [ "default" ];
      "pkg_id3" = [ ];
    };
  };

  testRootPackageWithOptional = {
    expr = packageFeatures "pkg_root" [ "default" "optional_id2" ];
    expected = {
      # "default" dropped (pkg_root declares none); "optional_id2" is real.
      "pkg_root" = [ "optional_id2" ];
      "pkg_id1" = [ "default" ];
      # pkg_id2 declares no `default` feature, so `default-features = true` on
      # the edge enables nothing — matching Cargo (previously [ "default" ]).
      "pkg_id2" = [ ];
      "pkg_id3" = [ ];
    };
  };

  # Regression test: requesting "default" on a crate that declares no `default`
  # feature must not synthesize a phantom "default" — whether it arrives via a
  # default-features-on dependency edge (pkg_id3 below) or as a root feature
  # (testRootPackage above). Otherwise the crate forks from any path that
  # reaches it without "default".
  testDefaultFeaturesNoOpWhenUndeclared = {
    expr = packageFeatures "pkg_no_default_root" [ ];
    expected = {
      "pkg_no_default_root" = [ ];
      "pkg_id3" = [ ];
    };
  };

  testPackageWithFeatureClash = {
    expr = packageFeatures "pkg_with_feature_clash" [ ];
    expected = {
      "pkg_with_feature_clash" = [ ];
      "pkg_id1" = [ "default" "for_build" ];
    };
  };
}
