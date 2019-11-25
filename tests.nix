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
        pregeneratedBuild? null,
        callableBuild? pregeneratedBuild,
        derivationAttrPath? ["rootCrate"]}:
        let
            nixBuild = if pregeneratedBuild == callableBuild
                    then tools.generated {
                      name = "buildTest_test_${name}";
                      inherit src cargoToml;
                    }
                    else pkgs.callPackage (./. + "/${callableBuild}") {};
            derivation = if pregeneratedBuild == callableBuild then
                  (lib.attrByPath derivationAttrPath null nixBuild).build.override { inherit features; }
                  else nixBuild;
        in pkgs.stdenv.mkDerivation {
            name = "buildTest_test_${name}";
            phases = [ "buildPhase" ];
            buildInputs = [ derivation ];
            buildPhase = ''
              mkdir -p $out
              ${derivation.crateName} >$out/test.log
              echo grepping
              grep '${expectedOutput}' $out/test.log || {
                echo '${expectedOutput}' not found in:
                cat $out/test.log
                exit 23
              }
            '';
          };

     buildTestConfigs = [
         {
             name = "sample_project_bin";
             src = ./sample_projects/bin;
             expectedOutput = "Hello, world!";
         }

         {
             name = "sample_project_lib_and_bin";
             src = ./sample_projects/lib_and_bin;
             expectedOutput = "Hello, lib_and_bin!";
         }

         {
             name = "sample_project_bin_with_lib_dep";
             src = ./sample_projects;
             cargoToml = "bin_with_lib_dep/Cargo.toml";
             expectedOutput = "Hello, bin_with_lib_dep!";
         }

         {
             name = "sample_project_bin_with_default_features";
             src = ./sample_projects;
             cargoToml = "bin_with_default_features/Cargo.toml";
             expectedOutput = "Hello, bin_with_default_features!";
         }

         {
             name = "sample_project_bin_with_NON_default_features";
             src = ./sample_projects;
             cargoToml = "bin_with_default_features/Cargo.toml";
             features = ["default" "do_not_activate"];
             expectedOutput = "Hello, bin_with_default_features, do_not_activate!";
         }

         {
            name = "sample_project_bin_with_lib_git_dep";
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
            src = ./sample_projects/bin_with_rerenamed_lib_dep;
            expectedOutput = "Hello, bin_with_rerenamed_lib_dep!";
            pregeneratedBuild = "sample_projects/bin_with_rerenamed_lib_dep/Cargo.nix";
         }

         {
            name = "sample_project_cfg_test";
            src = ./sample_projects/cfg-test;
            cargoToml = "Cargo.toml";
            expectedOutput = "Hello, cfg-test!";
            pregeneratedBuild = "sample_projects/cfg-test/Cargo.nix";
         }

         {
            name = "sample_workspace";
            src = ./sample_workspace;
            expectedOutput = "Hello, with_tera!";
            pregeneratedBuild = "sample_workspace/Cargo.nix";
            derivationAttrPath = [ "workspaceMembers" "with_tera" ];
         }

         {
            name = "sample_project_numtest";
            src = ./sample_projects/numtest;
            pregeneratedBuild = "sample_projects/numtest/Cargo.nix";
            expectedOutput = "Hello from numtest, world!";
         }

         {
            name = "sample_project_with_problematic_crates";
            src = ./sample_projects/with_problematic_crates;
            pregeneratedBuild = "sample_projects/with_problematic_crates/Cargo.nix";
            expectedOutput = "Hello, with_problematic_crates!";
         }

         {
            name = "sample_project_bin_with_git_submodule_dep";
            src = ./sample_projects/bin_with_git_submodule_dep;
            pregeneratedBuild = "sample_projects/bin_with_git_submodule_dep/Cargo.nix";
            callableBuild = "sample_projects/bin_with_git_submodule_dep/default.nix";
            expectedOutput = "Hello world from with_git_submodule_dep!";
         }
     ];

   buildTestDerivationAttrSet = let buildTestDerivations =
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

     sample_project_bin_with_deprecated_alias =
        let bin_build = (tools.generated {
            name = "sample_project_bin_with_deprecated_alias";
            src = ./sample_projects/bin;
        }).root_crate;
        in pkgs.stdenv.mkDerivation {
            name = "test_sample_project_bin";
            phases = [ "buildPhase" ];
            buildInputs = [ bin_build ];
            buildPhase = ''
              mkdir -p $out
              hello_world_bin >$out/test.log
              echo grepping
              grep 'Hello, world!' $out/test.log
        '';
      };

     sample_workspace_with_deprecated_alias =
        let bin_build = (pkgs.callPackage ./sample_workspace/Cargo.nix {})
            .workspace_members.with_tera;
        in pkgs.stdenv.mkDerivation {
            name = "test_sample_workspace_bin_with_deprecated_alias";
            phases = [ "buildPhase" ];
            buildInputs = [ bin_build ];
            buildPhase = ''
              mkdir -p $out
              with_tera >$out/test.log
              echo grepping
              grep 'Hello, with_tera!' $out/test.log || {
                echo "Unexpected output: "
                cat $out/test.log
                exit 1
              }
        '';
      };

    inherit buildTestConfigs;
    # TODO: File bug for cargo that it does an index fetch if fetching git package
    # even when lock file already exists
    # TODO: Make cargo proposal for modular builds.
} // buildTestDerivationAttrSet
