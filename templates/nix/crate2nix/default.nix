#
  # crate2nix/default.nix (excerpt start)
  # {#
{lib, buildRustCrate, crates? {}}:
rec {
  # #}

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

  /* Returns a buildRustCrate derivation for the given packageId and features. */
  buildRustCrateWithFeatures = { crateConfigs? crates, packageId, features } @ args:
    assert (builtins.isAttrs crateConfigs);
    assert (builtins.isString packageId);
    assert (builtins.isList features);

    let mergedFeatures = mergePackageFeatures args;
        buildByPackageId = packageId:
          let features = mergedFeatures."${packageId}" or [];
              crateConfig = crateConfigs."${packageId}";
              dependencies =
                dependencyDerivations buildByPackageId features (crateConfig.dependencies or {});
              buildDependencies =
                dependencyDerivations buildByPackageId features (crateConfig.buildDependencies or {});
          in buildRustCrate (crateConfig // { inherit features dependencies buildDependencies; });
    in buildByPackageId packageId;

  /* Returns the actual derivations for the given dependencies.
  */
  dependencyDerivations = buildByPackageId: features: dependencies:
    assert (builtins.isFunction buildByPackageId);
    assert (builtins.isList features);
    assert (builtins.isAttrs dependencies);

    let enabledDependencies =
          lib.filterAttrs
            (depName: dep:
              builtins.isString dep
              || dep.target or true
              && (!(dep.optional or false) || builtins.elem depName features))
            dependencies;
        depDerivation = dependencyName: dependency:
          buildByPackageId (dependencyPackageId dependency);
    in builtins.attrValues (lib.mapAttrs depDerivation enabledDependencies);

  /* Returns the feature configuration by package id for the given input crate. */
  mergePackageFeatures = {crateConfigs ? crates, packageId, features} @ args:
    assert (builtins.isAttrs crateConfigs);
    assert (builtins.isString packageId);
    assert (builtins.isList features);

    let packageFeatures = listOfPackageFeatures args;
        byPackageId = {packageId, features}: { "${packageId}" = features; };
        allByPackageId = builtins.map byPackageId packageFeatures;
    in assert (builtins.isList allByPackageId);
      lib.foldAttrs (f1: f2: (sortedUnique (f1 ++ f2))) [] allByPackageId;

  listOfPackageFeatures = {crateConfigs ? crates, packageId, features} @ args:
    assert (builtins.isAttrs crateConfigs);
    assert (builtins.isString packageId);
    assert (builtins.isList features);

    let
        crateConfig = crateConfigs."${packageId}" or (builtins.throw "Package not found: ${packageId}");
        expandedFeatures = expandFeatures (crateConfig.features or {}) features;
        depWithResolvedFeatures = dependencyName: dependency:
          let packageId = dependencyPackageId dependency;
          in { inherit packageId; features = dependencyFeatures expandedFeatures dependencyName dependency; };
        resolveDependencies = dependencies:
          assert (builtins.isAttrs dependencies);
          let directDependencies =
            builtins.attrValues
              (lib.mapAttrs depWithResolvedFeatures (filterEnabledDependencies dependencies expandedFeatures));
          in builtins.concatMap
            ({packageId, features}: listOfPackageFeatures { inherit crateConfigs packageId features; })
            directDependencies;
        resolvedDependencies = lib.concatMap
          resolveDependencies
          [
            (crateConfig.dependencies or {})
            (crateConfig.buildDependencies or {})
          ];
    in [{inherit packageId; features = expandedFeatures;}] ++ resolvedDependencies;

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

  sortedUnique = features:
    let outFeaturesSet = lib.foldl (set: feature: set // {"${feature}" = 1;} ) {} features;
        outFeaturesUnique = builtins.attrNames outFeaturesSet;
    in builtins.sort (a: b: a < b) outFeaturesUnique;

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

  #
  # crate2nix/default.nix (excerpt end)
  # {#
}
  # -#}
