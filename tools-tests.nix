let
  pkgs = import ./nix/nixpkgs.nix { };
  lib = pkgs.lib;
  tools = pkgs.callPackage ./tools.nix { };
  inherit (tools) internal;
in
{
  # hasWorkspaceTrue tests

  testHasWorkspaceTrue_directTrue = {
    expr = internal.hasWorkspaceTrue { workspace = true; };
    expected = true;
  };

  testHasWorkspaceTrue_directFalse = {
    expr = internal.hasWorkspaceTrue { workspace = false; };
    expected = false;
  };

  testHasWorkspaceTrue_noWorkspaceField = {
    expr = internal.hasWorkspaceTrue { version = "1.0"; };
    expected = false;
  };

  testHasWorkspaceTrue_nestedInDependencies = {
    expr = internal.hasWorkspaceTrue {
      dependencies = {
        serde = { workspace = true; };
      };
    };
    expected = true;
  };

  testHasWorkspaceTrue_deeplyNested = {
    expr = internal.hasWorkspaceTrue {
      target = {
        "cfg(unix)" = {
          dependencies = {
            libc = { workspace = true; };
          };
        };
      };
    };
    expected = true;
  };

  testHasWorkspaceTrue_inList = {
    expr = internal.hasWorkspaceTrue [
      { version = "1.0"; }
      { workspace = true; }
    ];
    expected = true;
  };

  testHasWorkspaceTrue_emptyAttrset = {
    expr = internal.hasWorkspaceTrue { };
    expected = false;
  };

  testHasWorkspaceTrue_string = {
    expr = internal.hasWorkspaceTrue "1.0";
    expected = false;
  };

  # resolveDependency tests

  testResolveDependency_simpleWorkspaceTrue = {
    expr = internal.resolveDependency
      { serde = { version = "1.0"; features = [ "derive" ]; }; }
      "serde"
      { workspace = true; };
    expected = { version = "1.0"; features = [ "derive" ]; };
  };

  testResolveDependency_workspaceWithStringVersion = {
    expr = internal.resolveDependency
      { serde = "1.0"; }
      "serde"
      { workspace = true; };
    expected = { version = "1.0"; };
  };

  testResolveDependency_workspaceWithOverrides = {
    expr = internal.resolveDependency
      { serde = { version = "1.0"; }; }
      "serde"
      { workspace = true; features = [ "derive" ]; };
    expected = { version = "1.0"; features = [ "derive" ]; };
  };

  testResolveDependency_nonWorkspaceDep = {
    expr = internal.resolveDependency
      { serde = { version = "1.0"; }; }
      "other"
      { version = "2.0"; };
    expected = { version = "2.0"; };
  };

  testResolveDependency_stringDep = {
    expr = internal.resolveDependency
      { }
      "serde"
      "1.0";
    expected = "1.0";
  };

  # resolveWorkspaceInheritance tests

  testResolveWorkspaceInheritance_packageVersion = {
    expr = internal.resolveWorkspaceInheritance
      {
        package = {
          name = "my-crate";
          version = { workspace = true; };
        };
      }
      {
        workspace = {
          package = { version = "1.2.3"; };
        };
      };
    expected = {
      package = {
        name = "my-crate";
        version = "1.2.3";
      };
    };
  };

  testResolveWorkspaceInheritance_dependencies = {
    expr = internal.resolveWorkspaceInheritance
      {
        package = { name = "my-crate"; };
        dependencies = {
          serde = { workspace = true; };
          other = { version = "2.0"; };
        };
      }
      {
        workspace = {
          dependencies = {
            serde = { version = "1.0"; features = [ "derive" ]; };
          };
        };
      };
    expected = {
      package = { name = "my-crate"; };
      dependencies = {
        serde = { version = "1.0"; features = [ "derive" ]; };
        other = { version = "2.0"; };
      };
    };
  };

  testResolveWorkspaceInheritance_targetDependencies = {
    expr = internal.resolveWorkspaceInheritance
      {
        package = { name = "my-crate"; };
        target = {
          "cfg(unix)" = {
            dependencies = {
              libc = { workspace = true; };
            };
          };
        };
      }
      {
        workspace = {
          dependencies = {
            libc = "0.2";
          };
        };
      };
    expected = {
      package = { name = "my-crate"; };
      target = {
        "cfg(unix)" = {
          dependencies = {
            libc = { version = "0.2"; };
          };
        };
      };
    };
  };

  testResolveWorkspaceInheritance_devDependencies = {
    expr = internal.resolveWorkspaceInheritance
      {
        package = { name = "my-crate"; };
        dev-dependencies = {
          tokio-test = { workspace = true; };
        };
      }
      {
        workspace = {
          dependencies = {
            tokio-test = { version = "0.4"; };
          };
        };
      };
    expected = {
      package = { name = "my-crate"; };
      dev-dependencies = {
        tokio-test = { version = "0.4"; };
      };
    };
  };

  testResolveWorkspaceInheritance_buildDependencies = {
    expr = internal.resolveWorkspaceInheritance
      {
        package = { name = "my-crate"; };
        build-dependencies = {
          cc = { workspace = true; };
        };
      }
      {
        workspace = {
          dependencies = {
            cc = "1.0";
          };
        };
      };
    expected = {
      package = { name = "my-crate"; };
      build-dependencies = {
        cc = { version = "1.0"; };
      };
    };
  };

  testResolveWorkspaceInheritance_preservesExtraFields = {
    expr = internal.resolveWorkspaceInheritance
      {
        package = { name = "my-crate"; };
        lib = { crate-type = [ "cdylib" ]; };
        features = { full = [ "serde" ]; };
      }
      {
        workspace = { };
      };
    expected = {
      package = { name = "my-crate"; };
      lib = { crate-type = [ "cdylib" ]; };
      features = { full = [ "serde" ]; };
    };
  };

  testResolveWorkspaceInheritance_noDependenciesSection = {
    expr = internal.resolveWorkspaceInheritance
      {
        package = { name = "my-crate"; version = "1.0.0"; };
      }
      {
        workspace = { };
      };
    expected = {
      package = { name = "my-crate"; version = "1.0.0"; };
    };
  };
}
