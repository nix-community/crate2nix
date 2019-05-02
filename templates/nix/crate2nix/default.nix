#
  # crate2nix/default.nix (excerpt start)
  # {#
{lib, stdenv, buildRustCrate, crates? {}}:
rec {
  # #}

  # Target (platform) data for conditional dependencies.
  # This corresponds to what buildRustCrate is setting.
  target = {
      unix = true;
      windows = false;

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

      # Filter out editor backup / swap files.
      lib.hasSuffix "~" baseName ||
      builtins.match "^\\.sw[a-z]$$" baseName != null ||
      builtins.match "^\\..*\\.sw[a-z]$$" baseName != null ||
      lib.hasSuffix ".tmp" baseName ||
      lib.hasSuffix ".bak" baseName
    );

  /* A restricted overridable version of  buildRustCrateWithFeaturesImpl. */
  buildRustCrateWithFeatures = {packageId, features}:
    lib.makeOverridable
      ({features}: buildRustCrateWithFeaturesImpl {inherit packageId features;})
      { inherit features; };

  /* Returns a buildRustCrate derivation for the given packageId and features. */
  buildRustCrateWithFeaturesImpl = { crateConfigs? crates, packageId, features } @ args:
    assert (builtins.isAttrs crateConfigs);
    assert (builtins.isString packageId);
    assert (builtins.isList features);

    let mergedFeatures = mergePackageFeatures args;
        buildByPackageId = packageId:
          let features = mergedFeatures."${packageId}" or [];
              crateConfig = lib.filterAttrs (n: v: n != "resolvedDefaultFeatures") crateConfigs."${packageId}";
              dependencies =
                dependencyDerivations buildByPackageId features (crateConfig.dependencies or {});
              buildDependencies =
                dependencyDerivations buildByPackageId features (crateConfig.buildDependencies or {});
          in buildRustCrate (crateConfig // { inherit features dependencies buildDependencies; });
    in buildByPackageId packageId;

  /* Returns the actual derivations for the given dependencies. */
  dependencyDerivations = buildByPackageId: features: dependencies:
    assert (builtins.isFunction buildByPackageId);
    assert (builtins.isList features);
    assert (builtins.isAttrs dependencies);

    let enabledDependencies = filterEnabledDependencies dependencies features;
        depDerivation = dependencyName: dependency:
        buildByPackageId (dependencyPackageId dependency);
    in builtins.attrValues (lib.mapAttrs depDerivation enabledDependencies);

  /* Returns differences between cargo default features and crate2nix default features.
   *
   * This is useful for verifying the feature resolution in crate2nix.
   */
  diffDefaultPackageFeatures = {crateConfigs ? crates, packageId}:
    assert (builtins.isAttrs crateConfigs);

    let prefixValues = prefix: lib.mapAttrs (n: v: { "${prefix}" = v; });
        mergedFeatures =
          prefixValues
            "crate2nix"
            (mergePackageFeatures {inherit crateConfigs packageId; features = ["default"]; });
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

  /* Returns the feature configuration by package id for the given input crate. */
  mergePackageFeatures = {crateConfigs ? crates, packageId, features} @ args:
    assert (builtins.isAttrs crateConfigs);
    assert (builtins.isString packageId);
    assert (builtins.isList features);

    let packageFeatures = listOfPackageFeatures args;
        grouped = lib.groupBy (x: x.packageId) packageFeatures;
    in lib.mapAttrs (n: v: sortedUnique (builtins.concatLists (builtins.map (v: v.features) v))) grouped;

  /* Returns a { packageId, features } attribute set for every package needed for building the
     package for the given packageId with the given features.

     Returns multiple, potentially conflicting attribute sets for dependencies that are reachable
     by multiple paths in the dependency tree.
  */
  listOfPackageFeatures = {crateConfigs ? crates, packageId, features, dependencyPath? [packageId]} @ args:
    assert (builtins.isAttrs crateConfigs);
    assert (builtins.isString packageId);
    assert (builtins.isList features);

    let
        crateConfig = crateConfigs."${packageId}" or (builtins.throw "Package not found: ${packageId}");
        expandedFeatures = expandFeatures (crateConfig.features or {}) features;

        depWithResolvedFeatures = dependencyName: dependency:
          let packageId = dependencyPackageId dependency;
              features = dependencyFeatures expandedFeatures dependencyName dependency;
          in { inherit packageId features; };

        resolveDependencies = path: dependencies:
          assert (builtins.isAttrs dependencies);

          let enabledDependencies = filterEnabledDependencies dependencies expandedFeatures;
              directDependencies =
                builtins.attrValues (lib.mapAttrs depWithResolvedFeatures enabledDependencies);
          in builtins.concatMap
            ({packageId, features}: listOfPackageFeatures {
              # This is purely for debugging.
              dependencyPath = dependencyPath ++ [path packageId];
              inherit crateConfigs packageId features;
            })
             directDependencies;

        resolvedDependencies = builtins.concatLists
          [
            (resolveDependencies "dependencies" (crateConfig.dependencies or {}))
            (resolveDependencies "buildDependencies" (crateConfig.buildDependencies or {}))
          ];

    in [{inherit packageId; features = expandedFeatures;}] ++ resolvedDependencies;

  /* Returns the enabled dependencies given the enabled features. */
  filterEnabledDependencies = dependencies: features:
    assert (builtins.isAttrs dependencies);
    assert (builtins.isList features);

    lib.filterAttrs
      (depName: dep:
        builtins.isString dep
        || dep.target or true
        && (!(dep.optional or false) || builtins.elem depName features))
      dependencies;

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

  /* The package ID of the given dependency. */
  dependencyPackageId = dependency: if builtins.isString dependency then dependency else dependency.package_id;

  /* Returns the actual dependencies for the given dependency. */
  dependencyFeatures = features: dependencyName: dependency:
    assert (builtins.isList features);
    assert (builtins.isString dependencyName);
    assert (builtins.isAttrs dependency || builtins.isString dependency);

    let defaultOrNil = if builtins.isString dependency || dependency.uses_default_features or true
                       then ["default"]
                       else [];
        explicitFeatures = if builtins.isString dependency then [] else dependency.features or [];
        additionalDependencyFeatures =
          let dependencyFeatures = builtins.filter (f: builtins.dirOf f == dependencyName) features;
          in builtins.map builtins.baseNameOf dependencyFeatures;
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
