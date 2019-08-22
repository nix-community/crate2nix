# 0.4.0 - 0.5.x (unreleased)

* Fix #10 - Changing default for `-n`/`--nixpkgs-path` to `"<nixpkgs>"` so that it works by default on NixOS AND Mac OS.
* Fix #11 - Adding optional dependencies if any of their features is enabled.

# 0.3.1 - 0.4.0

## Upgrading

Please change references to `root_crate` to `rootCrate.build` and references to `workspace_members.${crateName}` 
to `workspaceMembers.${crateName}.build`. The camel case attribute names are in line with the nixos style guide.
The `.build` suffix allows future versions of `crate2nix` to add other convenient features such as source tarball 
packages, docker image derivations, ... The old aliases still work but are deprecated. 

## Dynamic feature resolution

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
      in ...;
      ```
        
3. Or by overriding them on the rootCrate or workspaceMembers:

      ```nix
      let cargo_nix = callPackage ./Cargo.nix {};
          crate2nix = cargo_nix.rootCrate.build.override { features = ["default" "other"]; };
      in ...;
      ```
      
## Internal: nix test runner

For this release, I needed substantial amount of nix code so I created some nix unit tests. They are invoked by
`cargo test` like all other tests and live in the [./templates/nix/crate2nix/tests](./templates/nix/crate2nix/tests) 
directory.

## Feedback: What is needed for a 1.0 release?

I would really appreciate your thoughts. Please add comments to issue 
[#8](https://github.com/kolloch/crate2nix/issues/8).

# 0.3.0 - 0.3.1

## Bugfixes

* Issue [#5](https://github.com/kolloch/crate2nix/issues/5): Support `libPath` for proc-macro crates.

Thank you @nuxeh for reporting this bug! Again :)

# 0.2.1 - 0.3.0

## Bugfixes

* Issue [#4](https://github.com/kolloch/crate2nix/issues/4): Support for `libName` != `crateName`.

Thank you @nuxeh for reporting this bug!

## Support for dependencies with git sources

Example:

```toml
[dependencies]
"crate2nix" = { git = "https://github.com/kolloch/crate2nix" }
```

# 0.2.0 - 0.2.1

* Added comments to the generated nix build file to indicate which attributes are public and unlikely to change.

# 0.1.0 - 0.2.0

## Bugfixes

* While the command line help said that the "crate hashes" would be stored in a file called "crate-hashes.json", it
  actually used the file "crate_hashes.json" by default. This release uses the documented name which means that
  after the update `nix-prefetch-url` might run again.
* Issue [#3](https://github.com/kolloch/crate2nix/issues/3): Do not depend on local channel configuration for shell
  anymore. Instead, we use a recent nixos-unstable because we still need a fix that's not in stable.

## Workspace Support

If `crate2nix` is applied to a workspace, the resulting nix-file will contain a top-level "workspace_members" attribute 
set that refers the corresponding top-level crate derivations by name.

## Target-specific dependencies

"cfg(...)" expressions and target triplets such as "i686-pc-windows-gnu" are compiled into nix expressions. Support
should be reasonable but incomplete because e.g. it does not work for processor features. Please let me know if this 
causes problems for you! 

# 0.1.0

Initial public release.
