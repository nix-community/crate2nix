#
# crate2nix/default.nix (excerpt start)
#{#
{ pkgs
, lib
, stdenv
, buildRustCrate
, buildRustCrateForPkgs ? if buildRustCrate != null
  then lib.warn "`buildRustCrate` is deprecated, use `buildRustCrateForPkgs` instead" (_: buildRustCrate)
  else pkgs: pkgs.buildRustCrate
, defaultCrateOverrides
, strictDeprecation ? true
, crates ? { }
, rootFeatures ? [ ]
, targetFeatures ? [ ]
, release ? true
}:
rec {
  # #}

  /* Target (platform) data for conditional dependencies.
    This corresponds roughly to what buildRustCrate is setting.
  */
  makeDefaultTarget = platform: {
    unix = platform.isUnix;
    windows = platform.isWindows;
    fuchsia = true;
    test = false;

    /* We are choosing an arbitrary rust version to grab `lib` from,
      which is unfortunate, but `lib` has been version-agnostic the
      whole time so this is good enough for now.
    */
    os = pkgs.rust.lib.toTargetOs platform;
    arch = pkgs.rust.lib.toTargetArch platform;
    family = pkgs.rust.lib.toTargetFamily platform;
    env = "gnu";
    endian =
      if platform.parsed.cpu.significantByte.name == "littleEndian"
      then "little" else "big";
    pointer_width = toString platform.parsed.cpu.bits;
    vendor = platform.parsed.vendor.name;
    debug_assertions = false;
  };

  /* Filters common temp files and build files. */
  # TODO(pkolloch): Substitute with gitignore filter
  sourceFilter = name: type:
    let
      baseName = builtins.baseNameOf (builtins.toString name);
    in
      ! (
        # Filter out git
        baseName == ".gitignore"
        || (type == "directory" && baseName == ".git")

        # Filter out build results
        || (
          type == "directory" && (
            baseName == "target"
            || baseName == "_site"
            || baseName == ".sass-cache"
            || baseName == ".jekyll-metadata"
            || baseName == "build-artifacts"
          )
        )

        # Filter out nix-build result symlinks
        || (
          type == "symlink" && lib.hasPrefix "result" baseName
        )

        # Filter out IDE config
        || (
          type == "directory" && (
            baseName == ".idea" || baseName == ".vscode"
          )
        ) || lib.hasSuffix ".iml" baseName

        # Filter out nix build files
        || baseName == "Cargo.nix"

        # Filter out editor backup / swap files.
        || lib.hasSuffix "~" baseName
        || builtins.match "^\\.sw[a-z]$$" baseName != null
        || builtins.match "^\\..*\\.sw[a-z]$$" baseName != null
        || lib.hasSuffix ".tmp" baseName
        || lib.hasSuffix ".bak" baseName
        || baseName == "tests.nix"
      );

  /* A restricted overridable version of builtRustCratesWithFeatures. */
  buildRustCrateWithFeatures =
    { packageId
    , features ? rootFeatures
    , crateOverrides ? defaultCrateOverrides
    , buildRustCrateForPkgsFunc ? null
      # Available: [ "lib" "bin" ] or ["test" "bench" "example"]
    , buildKinds ? [ "lib" "bin" ]
    }:
    lib.makeOverridable
      (
        { features
        , crateOverrides
        , buildKinds
        }:
        let
          isDevBuild =
            let
              inherit (pkgs.buildRustCrateHelpers.kinds) isExample isTest isBench;
            in
              builtins.any (k: isBench k || isExample k || isTest k) buildKinds;

          buildRustCrateForPkgsFuncOverriden =
            if buildRustCrateForPkgsFunc != null
            then buildRustCrateForPkgsFunc
            else
              (
                if crateOverrides == pkgs.defaultCrateOverrides
                then buildRustCrateForPkgs
                else
                  pkgs: (buildRustCrateForPkgs pkgs).override {
                    defaultCrateOverrides = crateOverrides;
                  }
              );

          builtRustCrates = builtRustCratesWithFeatures {
            inherit packageId features buildKinds isDevBuild;
            buildRustCrateForPkgsFunc = buildRustCrateForPkgsFuncOverriden;
          };

        in
        builtRustCrates.crates.${packageId}
      )
      { inherit features crateOverrides buildKinds; };

  /* Returns an attr set with packageId mapped to the result of buildRustCrateForPkgsFunc
    for the corresponding crate.
  */
  builtRustCratesWithFeatures =
    { packageId
    , features
    , crateConfigs ? crates
    , buildRustCrateForPkgsFunc
    , buildKinds
    , isDevBuild
    , makeTarget ? makeDefaultTarget
    } @ args:
      assert (builtins.isAttrs crateConfigs);
      assert (builtins.isString packageId);
      assert (builtins.isList features);
      assert (builtins.isAttrs (makeTarget stdenv.hostPlatform));
      assert (builtins.isBool isDevBuild);
      assert (builtins.isList buildKinds);
      let
        rootPackageId = packageId;
        mergedFeatures = mergePackageFeatures
          (
            args // {
              inherit rootPackageId;
              # What does test do for target?
              target = makeTarget stdenv.hostPlatform // { test = false; };
            }
          );
        # Memoize built packages so that reappearing packages are only built once.
        builtByPackageIdByPkgs = mkBuiltByPackageIdByPkgs pkgs;
        mkBuiltByPackageIdByPkgs = pkgs:
          let
            self = {
              crates = lib.mapAttrs (packageId: value: buildByPackageIdForPkgsImpl self pkgs packageId) crateConfigs;
              target = makeTarget pkgs.stdenv.hostPlatform;
              build = mkBuiltByPackageIdByPkgs pkgs.buildPackages;
            };
          in
          self;
        buildByPackageIdForPkgsImpl = self: pkgs: packageId:
          let
            features = mergedFeatures."${packageId}" or [ ];
            crateConfig' = crateConfigs."${packageId}";
            crateConfig =
              builtins.removeAttrs crateConfig' [ "resolvedDefaultFeatures" "devDependencies" ];
            devDependencies =
              lib.optionals
                (isDevBuild && packageId == rootPackageId)
                (crateConfig'.devDependencies or [ ]);
            dependencies =
              dependencyDerivations {
                inherit features;
                inherit (self) target;
                buildByPackageId = depPackageId:
                  # proc_macro crates must be compiled for the build architecture
                  if crateConfigs.${depPackageId}.procMacro or false
                  then self.build.crates.${depPackageId}
                  else self.crates.${depPackageId};
                dependencies =
                  (crateConfig.dependencies or [ ])
                  ++ devDependencies;
              };
            buildDependencies =
              dependencyDerivations {
                inherit features;
                inherit (self.build) target;
                buildByPackageId = depPackageId:
                  self.build.crates.${depPackageId};
                dependencies = crateConfig.buildDependencies or [ ];
              };
            dependenciesWithRenames =
              let
                buildDeps = filterEnabledDependencies {
                  inherit features;
                  inherit (self) target;
                  dependencies = crateConfig.dependencies or [ ] ++ devDependencies;
                };
                hostDeps = filterEnabledDependencies {
                  inherit features;
                  inherit (self.build) target;
                  dependencies = crateConfig.buildDependencies or [ ];
                };
              in
              lib.filter (d: d ? "rename") (hostDeps ++ buildDeps);
            # Crate renames have the form:
            #
            # {
            #    crate_name = [
            #       { version = "1.2.3"; rename = "crate_name01"; }
            #    ];
            #    # ...
            # }
            crateRenames =
              let
                grouped =
                  lib.groupBy
                    (dependency: dependency.name)
                    dependenciesWithRenames;
                versionAndRename = dep:
                  let
                    package = crateConfigs."${dep.packageId}";
                  in
                  { inherit (dep) rename; version = package.version; };
              in
              lib.mapAttrs (name: choices: builtins.map versionAndRename choices) grouped;
          in
          buildRustCrateForPkgsFunc pkgs
            (
              crateConfig // {
                src = crateConfig.src or (
                  pkgs.fetchurl rec {
                    name = "${crateConfig.crateName}-${crateConfig.version}.tar.gz";
                    # https://www.pietroalbini.org/blog/downloading-crates-io/
                    # Not rate-limited, CDN URL.
                    url = "https://static.crates.io/crates/${crateConfig.crateName}/${crateConfig.crateName}-${crateConfig.version}.crate";
                    sha256 =
                      assert (lib.assertMsg (crateConfig ? sha256) "Missing sha256 for ${name}");
                      crateConfig.sha256;
                  }
                );
                extraRustcOpts = lib.lists.optional (targetFeatures != [ ]) "-C target-feature=${lib.concatMapStringsSep "," (x: "+${x}") targetFeatures}";
                buildKinds = if (packageId == rootPackageId) then buildKinds else [ "lib" ];
                inherit features dependencies buildDependencies crateRenames release;
              }
            );
      in
      builtByPackageIdByPkgs;

  /* Returns the actual derivations for the given dependencies. */
  dependencyDerivations =
    { buildByPackageId
    , features
    , dependencies
    , target
    }:
      assert (builtins.isList features);
      assert (builtins.isList dependencies);
      assert (builtins.isAttrs target);
      let
        enabledDependencies = filterEnabledDependencies {
          inherit dependencies features target;
        };
        depDerivation = dependency: buildByPackageId dependency.packageId;
      in
      map depDerivation enabledDependencies;

  /* Returns a sanitized version of val with all values substituted that cannot
    be serialized as JSON.
  */
  sanitizeForJson = val:
    if builtins.isAttrs val
    then lib.mapAttrs (n: v: sanitizeForJson v) val
    else if builtins.isList val
    then builtins.map sanitizeForJson val
    else if builtins.isFunction val
    then "function"
    else val;

  /* Returns various tools to debug a crate. */
  debugCrate = { packageId, target ? makeDefaultTarget stdenv.hostPlatform }:
    assert (builtins.isString packageId);
    let
      debug = rec {
        # The built tree as passed to buildRustCrate.
        buildTree = buildRustCrateWithFeatures {
          buildRustCrateForPkgsFunc = _: lib.id;
          inherit packageId;
        };
        sanitizedBuildTree = sanitizeForJson buildTree;
        dependencyTree = sanitizeForJson
          (
            buildRustCrateWithFeatures {
              buildRustCrateForPkgsFunc = _: crate: {
                "01_crateName" = crate.crateName or false;
                "02_features" = crate.features or [ ];
                "03_dependencies" = crate.dependencies or [ ];
              };
              inherit packageId;
            }
          );
        mergedPackageFeatures = mergePackageFeatures {
          features = rootFeatures;
          inherit packageId target;
        };
        diffedDefaultPackageFeatures = diffDefaultPackageFeatures {
          inherit packageId target;
        };
      };
    in
    { internal = debug; };

  /* Returns differences between cargo default features and crate2nix default
    features.

    This is useful for verifying the feature resolution in crate2nix.
  */
  diffDefaultPackageFeatures =
    { crateConfigs ? crates
    , packageId
    , target
    }:
      assert (builtins.isAttrs crateConfigs);
      let
        prefixValues = prefix: lib.mapAttrs (n: v: { "${prefix}" = v; });
        mergedFeatures =
          prefixValues
            "crate2nix"
            (mergePackageFeatures { inherit crateConfigs packageId target; features = [ "default" ]; });
        configs = prefixValues "cargo" crateConfigs;
        combined = lib.foldAttrs (a: b: a // b) { } [ mergedFeatures configs ];
        onlyInCargo =
          builtins.attrNames
            (lib.filterAttrs (n: v: !(v ? "crate2nix") && (v ? "cargo")) combined);
        onlyInCrate2Nix =
          builtins.attrNames
            (lib.filterAttrs (n: v: (v ? "crate2nix") && !(v ? "cargo")) combined);
        differentFeatures = lib.filterAttrs
          (
            n: v:
              (v ? "crate2nix")
              && (v ? "cargo")
              && (v.crate2nix.features or [ ]) != (v."cargo".resolved_default_features or [ ])
          )
          combined;
      in
      builtins.toJSON {
        inherit onlyInCargo onlyInCrate2Nix differentFeatures;
      };

  /* Returns an attrset mapping packageId to the list of enabled features.

    If multiple paths to a dependency enable different features, the
    corresponding feature sets are merged. Features in rust are additive.
  */
  mergePackageFeatures =
    { crateConfigs ? crates
    , packageId
    , rootPackageId ? packageId
    , features ? rootFeatures
    , dependencyPath ? [ crates.${packageId}.crateName ]
    , featuresByPackageId ? { }
    , target
      # Adds devDependencies to the crate with rootPackageId.
    , isDevBuild ? false
    , ...
    } @ args:
      assert (builtins.isAttrs crateConfigs);
      assert (builtins.isString packageId);
      assert (builtins.isString rootPackageId);
      assert (builtins.isList features);
      assert (builtins.isList dependencyPath);
      assert (builtins.isAttrs featuresByPackageId);
      assert (builtins.isAttrs target);
      assert (builtins.isBool isDevBuild);
      let
        crateConfig = crateConfigs."${packageId}" or (builtins.throw "Package not found: ${packageId}");
        expandedFeatures = expandFeatures (crateConfig.features or { }) features;
        enabledFeatures = enableFeatures (crateConfig.dependencies or [ ]) expandedFeatures;
        depWithResolvedFeatures = dependency:
          let
            packageId = dependency.packageId;
            features = dependencyFeatures enabledFeatures dependency;
          in
          { inherit packageId features; };
        resolveDependencies = cache: path: dependencies:
          assert (builtins.isAttrs cache);
          assert (builtins.isList dependencies);
          let
            enabledDependencies = filterEnabledDependencies {
              inherit dependencies target;
              features = enabledFeatures;
            };
            directDependencies = map depWithResolvedFeatures enabledDependencies;
            foldOverCache = op: lib.foldl op cache directDependencies;
          in
          foldOverCache
            (
              cache: { packageId, features }:
                let
                  cacheFeatures = cache.${packageId} or [ ];
                  combinedFeatures = sortedUnique (cacheFeatures ++ features);
                in
                if cache ? ${packageId} && cache.${packageId} == combinedFeatures
                then cache
                else
                  mergePackageFeatures {
                    features = combinedFeatures;
                    featuresByPackageId = cache;
                    inherit crateConfigs packageId target isDevBuild rootPackageId;
                  }
            );
        cacheWithSelf =
          let
            cacheFeatures = featuresByPackageId.${packageId} or [ ];
            combinedFeatures = sortedUnique (cacheFeatures ++ enabledFeatures);
          in
          featuresByPackageId // {
            "${packageId}" = combinedFeatures;
          };
        cacheWithDependencies =
          resolveDependencies cacheWithSelf "dep"
            (
              crateConfig.dependencies or [ ]
              ++ lib.optionals
                (isDevBuild && packageId == rootPackageId)
                (crateConfig.devDependencies or [ ])
            );
        cacheWithAll =
          resolveDependencies
            cacheWithDependencies "build"
            (crateConfig.buildDependencies or [ ]);
      in
      cacheWithAll;

  /* Returns the enabled dependencies given the enabled features. */
  filterEnabledDependencies = { dependencies, features, target }:
    assert (builtins.isList dependencies);
    assert (builtins.isList features);
    assert (builtins.isAttrs target);

    lib.filter
      (
        dep:
        let
          targetFunc = dep.target or (features: true);
        in
        targetFunc { inherit features target; }
        && (
          !(dep.optional or false)
          || builtins.any (doesFeatureEnableDependency dep) features
        )
      )
      dependencies;

  /* Returns whether the given feature should enable the given dependency. */
  doesFeatureEnableDependency = dependency: feature:
    let
      name = dependency.rename or dependency.name;
      prefix = "${name}/";
      len = builtins.stringLength prefix;
      startsWithPrefix = builtins.substring 0 len feature == prefix;
    in
    feature == name || feature == "dep:" + name || startsWithPrefix;

  /* Returns the expanded features for the given inputFeatures by applying the
    rules in featureMap.

    featureMap is an attribute set which maps feature names to lists of further
    feature names to enable in case this feature is selected.
  */
  expandFeatures = featureMap: inputFeatures:
    assert (builtins.isAttrs featureMap);
    assert (builtins.isList inputFeatures);
    let
      expandFeature = feature:
        assert (builtins.isString feature);
        [ feature ] ++ (expandFeatures featureMap (featureMap."${feature}" or [ ]));
      outFeatures = lib.concatMap expandFeature inputFeatures;
    in
    sortedUnique outFeatures;

  /* This function adds optional dependencies as features if they are enabled
    indirectly by dependency features. This function mimics Cargo's behavior
    described in a note at:
    https://doc.rust-lang.org/nightly/cargo/reference/features.html#dependency-features
  */
  enableFeatures = dependencies: features:
    assert (builtins.isList features);
    assert (builtins.isList dependencies);
    let
      additionalFeatures = lib.concatMap
        (
          dependency:
            assert (builtins.isAttrs dependency);
            let
              enabled = builtins.any (doesFeatureEnableDependency dependency) features;
            in
            if (dependency.optional or false) && enabled
            then [ (dependency.rename or dependency.name) ]
            else [ ]
        )
        dependencies;
    in
    sortedUnique (features ++ additionalFeatures);

  /*
    Returns the actual features for the given dependency.

    features: The features of the crate that refers this dependency.
  */
  dependencyFeatures = features: dependency:
    assert (builtins.isList features);
    assert (builtins.isAttrs dependency);
    let
      defaultOrNil =
        if dependency.usesDefaultFeatures or true
        then [ "default" ]
        else [ ];
      explicitFeatures = dependency.features or [ ];
      additionalDependencyFeatures =
        let
          name = dependency.rename or dependency.name;
          stripPrefixMatch = prefix: s:
            if lib.hasPrefix prefix s
            then lib.removePrefix prefix s
            else null;
          extractFeature = feature: lib.findFirst
            (f: f != null)
            null
            (map (prefix: stripPrefixMatch prefix feature) [
              (name + "/")
              (name + "?/")
            ]);
          dependencyFeatures = lib.filter (f: f != null) (map extractFeature features);
        in
        dependencyFeatures;
    in
    defaultOrNil ++ explicitFeatures ++ additionalDependencyFeatures;

  /* Sorts and removes duplicates from a list of strings. */
  sortedUnique = features:
    assert (builtins.isList features);
    assert (builtins.all builtins.isString features);
    let
      outFeaturesSet = lib.foldl (set: feature: set // { "${feature}" = 1; }) { } features;
      outFeaturesUnique = builtins.attrNames outFeaturesSet;
    in
    builtins.sort (a: b: a < b) outFeaturesUnique;

  deprecationWarning = message: value:
    if strictDeprecation
    then builtins.throw "strictDeprecation enabled, aborting: ${message}"
    else builtins.trace message value;

  #
  # crate2nix/default.nix (excerpt end)
  #{#
}
# -#}
