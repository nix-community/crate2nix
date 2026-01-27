---
title: Choosing enabled features
---

The enabled features for a crate now are resolved at build time! That means you can easily override them:

1. There is a "rootFeatures" argument to the generated build file which you can override when calling
   it from the command line:

      ```bash
      nix build -f ....nix --arg rootFeatures '["default" "other"]' rootCrate.build
      ```

2. Or when importing the build file with "callPackage":

      ```nix
      let cargo_nix = callPackage ./Cargo.nix { rootFeatures = ["default" "other"]; };
          crate2nix = cargo_nix.rootCrate.build;
      in ...
      ```

3. Or by overriding them on the rootCrate or workspaceMembers:

      ```nix
      let cargo_nix = callPackage ./Cargo.nix {};
          crate2nix = cargo_nix.rootCrate.build.override { features = ["default" "other"]; };
      in ...
      ```

Note that only dependencies for the default features are included in the build.
If you want full flexibility, you can use `crate2nix generate --all-features` to
generate the most general build file. If you want to strip down the generated
build file, you may want to use `crate2nix generate --no-default-features
--features "feature1 feature2"`.
