#
  # crate2nix/default.nix (excerpt start)
  # {#
{pkgs, lib, stdenv, buildRustCrate, defaultCrateOverrides, crates? {}, rootFeatures? []}:
rec {
  # #}

  # Target (platform) data for conditional dependencies.
  # This corresponds roughly to what buildRustCrate is setting.
  defaultTarget = {
      unix = true;
      windows = false;
      fuchsia = true;
      test = false;

      # This doesn't appear to be officially documented anywhere yet.
      # See https://github.com/rust-lang-nursery/rust-forge/issues/101.
      os = if stdenv.hostPlatform.isDarwin
        then "macos"
        else stdenv.hostPlatform.parsed.kernel.name;
      arch = stdenv.hostPlatform.parsed.cpu.name;
      family = "unix";
      env = "gnu";
      endian = if stdenv.hostPlatform.parsed.cpu.significantByte.name == "littleEndian" then "little" else "big";
      pointer_width = toString stdenv.hostPlatform.parsed.cpu.bits;
      vendor = stdenv.hostPlatform.parsed.vendor.name;
      debug_assertions = false;
  };

  /* Filters common temp files and build files */
  # TODO(pkolloch): Substitute with gitignore filter
  sourceFilter = name: type:
    let baseName = builtins.baseNameOf (builtins.toString name);
    in ! (
      # Filter out git
      baseName == ".gitignore" ||
      (type == "directory" && baseName == ".git" ) ||

      # Filter out build results
      (type == "directory" && (
        baseName == "target" ||
        baseName == "_site" ||
        baseName == ".sass-cache" ||
        baseName == ".jekyll-metadata" ||
        baseName == "build-artifacts"
        )) ||

      # Filter out nix-build result symlinks
      (type == "symlink" && lib.hasPrefix "result" baseName) ||

      # Filter out IDE config
      (type == "directory" && (
        baseName == ".idea" ||
        baseName == ".vscode"
        )) ||
      lib.hasSuffix ".iml" baseName ||

      # Filter out nix build files
      # lib.hasSuffix ".nix" baseName ||
      baseName == "Cargo.nix" ||

      # Filter out editor backup / swap files.
      lib.hasSuffix "~" baseName ||
      builtins.match "^\\.sw[a-z]$$" baseName != null ||
      builtins.match "^\\..*\\.sw[a-z]$$" baseName != null ||
      lib.hasSuffix ".tmp" baseName ||
      lib.hasSuffix ".bak" baseName ||
      baseName == "tests.nix"
    );

  /* A restricted overridable version of  buildRustCrateWithFeaturesImpl. */
  buildRustCrateWithFeatures =
    { packageId
    , features ? rootFeatures
    , crateOverrides ? defaultCrateOverrides
    , buildRustCrateFunc ? buildRustCrate
    , doTest ? false,
    }:
    lib.makeOverridable
      ({features, crateOverrides, doTest}:
        let builtRustCrates = builtRustCratesWithFeatures {
          inherit packageId features crateOverrides buildRustCrateFunc doTest;
        };
        in builtRustCrates.${packageId})
      { inherit features crateOverrides doTest; };

  /* Returns a buildRustCrate derivation for the given packageId and features. */
  builtRustCratesWithFeatures =
    { packageId
    , features
    , crateConfigs ? crates
    , crateOverrides
    , buildRustCrateFunc
    , doTest
    , target ? defaultTarget
    } @ args:
    assert (builtins.isAttrs crateConfigs);
    assert (builtins.isString packageId);
    assert (builtins.isList features);
    assert (builtins.isAttrs target);

    let mergedFeatures = mergePackageFeatures ( args // { inherit target; });
        mergedTestFeatures = mergePackageFeatures ( args // { target = target // { test = true; }; });

        buildByPackageId = packageId: buildByPackageIdImpl packageId false;
        buildTestByPackageId = packageId: buildByPackageIdImpl packageId true;

        # Memoize built packages so that reappearing packages are only built once.
        builtByPackageId =
          lib.mapAttrs (packageId: value: buildByPackageId packageId) crateConfigs;

        buildByPackageIdImpl = packageId: test:
          let
              features = let f = if !test then mergedFeatures else mergedTestFeatures; in
                f."${packageId}" or [];
              crateConfig = lib.filterAttrs (n: v: n != "resolvedDefaultFeatures") crateConfigs."${packageId}";
              dependencies =
                dependencyDerivations {
                  inherit builtByPackageId features target;
                  dependencies = crateConfig.dependencies or [];
                };
              buildDependencies =
                dependencyDerivations {
                  inherit builtByPackageId features target;
                  dependencies = crateConfig.buildDependencies or [];
                };
              dependenciesWithRenames =
                lib.filter (d: d ? "rename")
                  (crateConfig.buildDependencies or [] ++ crateConfig.dependencies or []);
              crateRenames =
                builtins.listToAttrs (map (d: { name = d.name; value = d.rename; }) dependenciesWithRenames);
          in buildRustCrateFunc (crateConfig // {
            src = crateConfig.src or (pkgs.fetchurl {
              name = "${crateConfig.crateName}-${crateConfig.version}.tar.gz";
              url = "https://crates.io/api/v1/crates/${crateConfig.crateName}/${crateConfig.version}/download";
              sha256 = crateConfig.sha256;
            });
            inherit features dependencies buildDependencies crateRenames;
          });
    in builtByPackageId;

  /* Returns the actual derivations for the given dependencies. */
  dependencyDerivations =
    { builtByPackageId
    , features
    , dependencies
    , target
    }:
    assert (builtins.isAttrs builtByPackageId);
    assert (builtins.isList features);
    assert (builtins.isList dependencies);

    let enabledDependencies = filterEnabledDependencies { inherit dependencies features target; };
        depDerivation = dependency: builtByPackageId.${dependency.packageId};
    in map depDerivation enabledDependencies;

  sanitizeForJson = val:
          if builtins.isAttrs val
          then lib.mapAttrs (n: v: sanitizeForJson v) val
          else if builtins.isList val
          then builtins.map sanitizeForJson val
          else if builtins.isFunction val
          then "function"
          else val;

  debugCrate = {packageId, target}:
    assert (builtins.isString packageId);

    rec {
        # The built tree as passed to buildRustCrate.
        buildTree = buildRustCrateWithFeatures {
            buildRustCrateFunc = lib.id;
            inherit packageId;
        };
        sanitizedBuildTree = sanitizeForJson buildTree;
        dependencyTree = sanitizeForJson (buildRustCrateWithFeatures {
            buildRustCrateFunc = crate: {
                "01_crateName" = crate.crateName or false;
                "02_features" = crate.features or [];
                "03_dependencies" = crate.dependencies or [];
            };
            inherit packageId;
        });
        mergedPackageFeatures = mergePackageFeatures { inherit packageId target; features = rootFeatures; };
        diffedDefaultPackageFeatures = diffDefaultPackageFeatures { inherit packageId;  features = rootFeatures; };
    };

  /* Returns differences between cargo default features and crate2nix default features.
   *
   * This is useful for verifying the feature resolution in crate2nix.
   */
  diffDefaultPackageFeatures =
    { crateConfigs ? crates
    , packageId
    , target
    }:
    assert (builtins.isAttrs crateConfigs);

    let prefixValues = prefix: lib.mapAttrs (n: v: { "${prefix}" = v; });
        mergedFeatures =
          prefixValues
            "crate2nix"
            (mergePackageFeatures {inherit crateConfigs packageId target; features = ["default"]; });
        configs = prefixValues "cargo" crateConfigs;
        combined = lib.foldAttrs (a: b: a // b) {} [ mergedFeatures configs ];
        onlyInCargo = builtins.attrNames (lib.filterAttrs (n: v: !(v ? "crate2nix" ) && (v ? "cargo")) combined);
        onlyInCrate2Nix = builtins.attrNames (lib.filterAttrs (n: v: (v ? "crate2nix" ) && !(v ? "cargo")) combined);
        differentFeatures = lib.filterAttrs
          (n: v:
          (v ? "crate2nix" )
          && (v ? "cargo")
          && (v.crate2nix.features or []) != (v."cargo".resolved_default_features or []))
          combined;
    in builtins.toJSON { inherit onlyInCargo onlyInCrate2Nix differentFeatures; };

  /* Returns the feature configuration by package id for the given input crate.

     Returns a { packageId, features } attribute set for every package needed for building the
     package for the given packageId with the given features.

     Returns multiple, potentially conflicting attribute sets for dependencies that are reachable
     by multiple paths in the dependency tree.
  */

  mergePackageFeatures = {
    crateConfigs ? crates,
    packageId,
    features ? rootFeatures,
    dependencyPath? [crates.${packageId}.crateName],
    featuresByPackageId? {},
    target,
    ...} @ args:
    assert (builtins.isAttrs crateConfigs);
    assert (builtins.isString packageId);
    assert (builtins.isList features);
    assert (builtins.isAttrs featuresByPackageId);

    let
        crateConfig = crateConfigs."${packageId}" or (builtins.throw "Package not found: ${packageId}");
        expandedFeatures = expandFeatures (crateConfig.features or {}) features;

        depWithResolvedFeatures = dependency:
          let packageId = dependency.packageId;
              features = dependencyFeatures expandedFeatures dependency;
          in { inherit packageId features; };

        resolveDependencies = cache: path: dependencies:
          assert (builtins.isAttrs cache);
          assert (builtins.isList dependencies);

          let enabledDependencies = filterEnabledDependencies {
                inherit dependencies target;
                features = expandedFeatures;
              };
              directDependencies = map depWithResolvedFeatures enabledDependencies;
              foldOverCache = op: lib.foldl op cache directDependencies;
          in foldOverCache
            (cache: {packageId, features}:
             let cacheFeatures = cache.${packageId} or [];
                 combinedFeatures = sortedUnique (cacheFeatures ++ features);
             in
             if cache ? ${packageId} && cache.${packageId} == combinedFeatures
             then cache
             else mergePackageFeatures {
                  # This is purely for debugging.
                  dependencyPath = dependencyPath ++ [path crateConfigs.${packageId}.crateName];
                  features = combinedFeatures;
                  featuresByPackageId = cache;
                  inherit crateConfigs packageId target;
                 });

        cacheWithSelf =
            let cacheFeatures = featuresByPackageId.${packageId} or [];
                combinedFeatures = sortedUnique (cacheFeatures ++ expandedFeatures);
            in featuresByPackageId // {
                ${packageId} = combinedFeatures;
            };

        cacheWithDependencies =
            resolveDependencies cacheWithSelf "dep" (crateConfig.dependencies or []);
        cacheWithAll =
            resolveDependencies cacheWithDependencies "build" (crateConfig.buildDependencies or []);

    in cacheWithAll;

  /* Returns the enabled dependencies given the enabled features. */
  filterEnabledDependencies = {dependencies, features, target}:
    assert (builtins.isList dependencies);
    assert (builtins.isList features);

    lib.filter
      (dep:
        let targetFunc = dep.target or (features: true);
        in targetFunc { inherit features target; }
           && (!(dep.optional or false) || builtins.any (doesFeatureEnableDependency dep) features))
      dependencies;

  /* Returns whether the given feature should enable the given dependency. */
  doesFeatureEnableDependency = { name, rename ? null, ...}: feature:
    let prefix = "${name}/";
        len = builtins.stringLength prefix;
        startsWithPrefix = builtins.substring 0 len feature == prefix;
    in feature == name || (rename != null && rename == feature) || startsWithPrefix;

  /* Returns the expanded features for the given inputFeatures by applying the rules in featureMap.

     featureMap is an attribute set which maps feature names to lists of further feature names to enable in case this
     feature is selected.
  */
  expandFeatures = featureMap: inputFeatures:
    assert (builtins.isAttrs featureMap);
    assert (builtins.isList inputFeatures);

    let expandFeature = feature:
          assert (builtins.isString feature);
          [feature] ++ (expandFeatures featureMap (featureMap."${feature}" or []));
        outFeatures = builtins.concatMap expandFeature inputFeatures;
    in sortedUnique outFeatures;

  /*
   * Returns the actual dependencies for the given dependency.
   *
   * features: The features of the crate that refers this dependency.
   */
  dependencyFeatures = features: dependency:
    assert (builtins.isList features);
    assert (builtins.isAttrs dependency);

    let defaultOrNil = if dependency.usesDefaultFeatures or true
                       then ["default"]
                       else [];
        explicitFeatures = dependency.features or [];
        additionalDependencyFeatures =
          let dependencyPrefix = dependency.name+"/";
              dependencyFeatures =
                builtins.filter (f: lib.hasPrefix dependencyPrefix f) features;
          in builtins.map (lib.removePrefix dependencyPrefix) dependencyFeatures;
    in
      defaultOrNil ++ explicitFeatures ++ additionalDependencyFeatures;

  /* Sorts and removes duplicates from a list of strings. */
  sortedUnique = features:
    assert (builtins.isList features);
    assert (builtins.all builtins.isString features);

    let outFeaturesSet = lib.foldl (set: feature: set // {"${feature}" = 1;} ) {} features;
        outFeaturesUnique = builtins.attrNames outFeaturesSet;
    in builtins.sort (a: b: a < b) outFeaturesUnique;

  #
  # crate2nix/default.nix (excerpt end)
  # {#
}
  # -#}
