---
title: Creating a release
---

:::tip[Target Audience]
Maintainers of crate2nix

:::

- [ ] Update flake dependencies: `nix flake update`
- [ ] `./run_tests.sh`
- [ ] Verify build on Mac OS X
- [ ] Verify that generated output looks nice
- [ ] Verify that CHANGELOG is up-to-date
- [ ] Verify that new features are documented
- [ ] Bump version in Cargo.toml: `cargo release --no-push Major/Minor/Alpha/Release/RC`
- [ ] `./run_tests.sh` to regenerate sources after version bump
- [ ] Tag version e.g. `0.13.0` (without leading v)
- [ ] Push
- [ ] `cargo publish`
- [ ] In `flake.nix`: bump input `crate2nix-stable`
- [ ] Create release from tag
