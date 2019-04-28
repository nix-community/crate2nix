{lib}:

rec {
  /* Returns the expanded features for the given inputFeatures by applying the rules in featureMap.

     featureMap is an attribute set which maps feature names to lists of further feature names to enable in case this
     feature is selected.
  */
  expandFeatures = featureMap: inputFeatures:
    assert (builtins.isAttrs featureMap);
    assert (builtins.isList inputFeatures);

    let expandFeature = feature:
          assert (builtins.isString feature);
          [feature] ++ (expandFeatures featureMap (featureMap.${feature} or []));
        outFeatures = builtins.concatMap expandFeature inputFeatures;
        outFeaturesSet = lib.foldl (set: feature: set // {${feature} = 1;} ) {} outFeatures;
        outFeaturesUnique = builtins.attrNames outFeaturesSet;
    in builtins.sort (a: b: a < b) outFeaturesUnique;

  /* Returns the actual dependencies for the given dependency. */
  dependencyFeatures = features: dependencyName: dependency:
    assert (builtins.isList features);
    assert (builtins.isString dependencyName);
    assert (builtins.isAttrs dependency || builtins.isString dependency);

    let defaultOrNil = if builtins.isString dependency || dependency.uses_default_features or true
                       then ["default"]
                       else [];
        additionalDependencyFeatures =
          let dependencyFeatures = builtins.filter (f: builtins.dirOf f == dependencyName) features;
          in builtins.map builtins.baseNameOf dependencyFeatures;
    in
      defaultOrNil ++ additionalDependencyFeatures;

  /* Returns the actual derivations for the given enabled features and dependencies.

     `crateDerivations` is expected to map `package IDs` to `buildRustCrate` derivations.
  */
  dependencyDerivations = crateDerivations: features: dependencies:
    assert (builtins.isAttrs crateDerivations);
    assert (builtins.isAttrs dependencies);
    assert (builtins.isList features);

    let enabledDependencies =
          lib.filterAttrs
            (depName: dep:
              builtins.isString dep
              || dep.target_cfg or true
              && (!(dep.optional or false) || builtins.elem depName features))
            dependencies;
        depDerivation = dependencyName: dependency:
          let packageId = if builtins.isString dependency then dependency else dependency.package_id;
          in crateDerivations.${packageId} {
            features = dependencyFeatures features dependencyName dependency;
          };
        derivations = builtins.attrValues (lib.mapAttrs depDerivation enabledDependencies);
    in
      lib.sort (a: b: a.crateName < b.crateName) derivations;
}
