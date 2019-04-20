# 0.2.0.beta.0

## Bugfixes

* While the command line help said that the "crate hashes" would be stored in a file called "crate-hashes.json", it
  actually used the file "crate_hashes.json" by default. This release uses the documented name which means that
  after the update `nix-prefetch-url` might run again.
* Issue [#3](https://github.com/kolloch/crate2nix/issues/3): Do not depend on local channel configuration for shell
  anymore. Instead, we use a recent nixos-unstable because we still need a fix that's not in stable.

## Workspace Support

If `crate2nix` is applied to a workspace, the resulting nix-file will contain a top-level "workspace_members" attribute 
set that refers the corresponding top-level crate derivations by name.

# 0.1.0

Initial public release.
