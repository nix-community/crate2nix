{ pkgs? import ./nixpkgs.nix { config = {}; }, lib ? pkgs.lib}:

let crate2nix = pkgs.callPackage ./default.nix {};
    nodes = {
      dev = {pkgs, ...}: {
        environment.systemPackages = [crate2nix];
      };
    };
    tools = pkgs.callPackage ./tools.nix {};
    buildTest = {
        name, src, cargoToml? "Cargo.toml", features? ["default"],
        expectedOutput,
        expectedTestOutputs ? [],
        pregeneratedBuild? null,
        additionalCargoNixArgs? [],
        customBuild? null,
        derivationAttrPath? ["rootCrate"],
      }:
        let
          generatedBuild =
            if builtins.isNull pregeneratedBuild
              then
                tools.generate {
                  name = "buildTest_test_${name}";
                  inherit src cargoToml additionalCargoNixArgs;
                }
              else
                ./. + "/${pregeneratedBuild}";
          derivation =
            if builtins.isNull customBuild
              then
                (lib.attrByPath derivationAttrPath null (pkgs.callPackage generatedBuild {})).build.override {
                  inherit features;
                }
            else
                pkgs.callPackage (./. + "/${customBuild}") {
                  inherit generatedBuild;
                };
        in
          assert lib.length expectedTestOutputs > 0 -> derivation ? test;
          pkgs.stdenv.mkDerivation {
            name = "buildTest_${name}";
            phases = [ "buildPhase" ];
            buildInputs = [ derivation ];
            buildPhase = ''
              mkdir -p $out
              ${derivation.crateName} >$out/run.log
              echo grepping
              grep '${expectedOutput}' $out/run.log || {
                echo '${expectedOutput}' not found in:
                cat $out/run.log
                exit 23
              }

              ${lib.optionalString (lib.length expectedTestOutputs > 0) ''
                cp ${derivation.test} $out/tests.log
              ''}
              ${lib.concatMapStringsSep "\n" (output: ''
                grep '${output}' $out/tests.log || {
                  echo '${output}' not found in:
                  cat $out/tests.log
                  exit 23
                }
              '') expectedTestOutputs}
            '';
          };

     buildTestConfigs = [
         {
             name = "bin";
             src = ./sample_projects/bin;
             expectedOutput = "Hello, world!";
         }

         {
             name = "lib_and_bin";
             src = ./sample_projects/lib_and_bin;
             expectedOutput = "Hello, lib_and_bin!";
         }

         {
             name = "bin_with_lib_dep";
             src = ./sample_projects;
             cargoToml = "bin_with_lib_dep/Cargo.toml";
             expectedOutput = "Hello, bin_with_lib_dep!";
         }

         {
             name = "bin_with_default_features";
             src = ./sample_projects;
             cargoToml = "bin_with_default_features/Cargo.toml";
             expectedOutput = "Hello, bin_with_default_features!";
         }

         {
             name = "bin_with_NON_default_features";
             src = ./sample_projects;
             cargoToml = "bin_with_default_features/Cargo.toml";
             features = ["default" "do_not_activate"];
             expectedOutput = "Hello, bin_with_default_features, do_not_activate!";
         }

         {
            name = "bin_with_lib_git_dep";
            src = ./sample_projects/bin_with_lib_git_dep;
            expectedOutput = "Hello world from bin_with_lib_git_dep!";
            pregeneratedBuild = "sample_projects/bin_with_lib_git_dep/Cargo.nix";
         }

         {
            name = "bin_with_git_branch_dep";
            src = ./sample_projects/bin_with_git_branch_dep;
            expectedOutput = "Hello world from bin_with_git_branch_dep!";
            pregeneratedBuild = "sample_projects/bin_with_git_branch_dep/Cargo.nix";
         }

         {
            name = "bin_with_rerenamed_lib_dep";
            src = ./sample_projects;
            cargoToml = "bin_with_rerenamed_lib_dep/Cargo.toml";
            expectedOutput = "Hello, bin_with_rerenamed_lib_dep!";
         }

         {
            name = "cfg_test";
            src = ./sample_projects/cfg-test;
            cargoToml = "Cargo.toml";
            expectedOutput = "Hello, cfg-test!";
         }

         {
            name = "futures_compat_test";
            src = ./sample_projects/futures_compat;
            cargoToml = "Cargo.toml";
            expectedOutput = "Hello, futures_compat!";
         }

         {
            name = "cfg_test-with-tests";
            src = ./sample_projects/cfg-test;
            cargoToml = "Cargo.toml";
            expectedOutput = "Hello, cfg-test!";
            expectedTestOutputs = [
              "test echo_foo_test ... ok"
              "test lib_test ... ok"
            ];
            customBuild = "sample_projects/cfg-test/test.nix";
         }

         {
             name = "renamed_build_deps";
             src = ./sample_projects/renamed_build_deps;
             expectedOutput = "Hello, renamed_build_deps!";
         }

         {
            name = "sample_workspace";
            src = ./sample_workspace;
            expectedOutput = "Hello, with_tera!";
            derivationAttrPath = [ "workspaceMembers" "with_tera" ];
         }

         {
            name = "numtest";
            src = ./sample_projects/numtest;
            expectedOutput = "Hello from numtest, world!";
         }

         {
            name = "dependency_issue_65_all_features";
            additionalCargoNixArgs = ["--all-features"];
            src = ./sample_projects/dependency_issue_65;
            customBuild = "sample_projects/dependency_issue_65/default.nix";
            expectedOutput = "Hello from dependency_issue_65, world!";
         }

         {
            name = "dependency_issue_65_sqlite_feature";
            additionalCargoNixArgs = ["--features" "sqlite"];
            src = ./sample_projects/dependency_issue_65;
            customBuild = "sample_projects/dependency_issue_65/default.nix";
            expectedOutput = "Hello from dependency_issue_65, world!";
         }

         {
            name = "numtest_new_cargo_lock";
            src = ./sample_projects/numtest_new_cargo_lock;
            expectedOutput = "Hello from numtest, world!";
         }

         {
            name = "with_problematic_crates";
            src = ./sample_projects/with_problematic_crates;
            expectedOutput = "Hello, with_problematic_crates!";
         }

         {
            name = "bin_with_git_submodule_dep";
            src = ./sample_projects/bin_with_git_submodule_dep;
            pregeneratedBuild = "sample_projects/bin_with_git_submodule_dep/Cargo.nix";
            customBuild = "sample_projects/bin_with_git_submodule_dep/default.nix";
            expectedOutput = "Hello world from with_git_submodule_dep!";
         }

         {
            name = "cdylib";
            src = ./sample_projects/cdylib;
            customBuild = "sample_projects/cdylib/test.nix";
            expectedOutput = "cdylib test";
         }
     ];

   buildTestDerivationAttrSet = 
    let 
      buildTestDerivations =
      builtins.map
        (c: {name = c.name;  value = buildTest c;})
        buildTestConfigs;
    in builtins.listToAttrs buildTestDerivations;

in {
     help = pkgs.stdenv.mkDerivation {
        name = "help";
        phases = [ "buildPhase" ];
        buildPhase = ''
        mkdir -p $out
        ${crate2nix}/bin/crate2nix help >$out/crate2nix.log
        echo grepping
        grep USAGE $out/crate2nix.log
        '';
      };

     fail = pkgs.stdenv.mkDerivation {
        name = "fail";
        phases = [ "buildPhase" ];
        buildPhase = ''
        mkdir -p $out
        ${crate2nix}/bin/crate2nix 2>$out/crate2nix.log \
            && exit 23 || echo expect error
        echo grepping
        grep USAGE $out/crate2nix.log
        '';
      };

     bin_with_deprecated_alias =
        let bin_build = (tools.generated {
            name = "bin_with_deprecated_alias";
            src = ./sample_projects/bin;
        }).root_crate;
        in pkgs.stdenv.mkDerivation {
            name = "test_bin";
            phases = [ "buildPhase" ];
            buildInputs = [ bin_build ];
            buildPhase = ''
              mkdir -p $out
              hello_world_bin >$out/test.log
              echo grepping
              grep 'Hello, world!' $out/test.log
        '';
      };

    inherit buildTestConfigs;
} // buildTestDerivationAttrSet
