# Creating a release

Release checklist for crate2nix maintainers.

- [ ] Update flake dependencies: `nix flake update`
- [ ] `./run_tests.sh`
- [ ] Verify build on Mac OS X
- [ ] Verify that generated output looks nice
- [ ] Verify that CHANGELOG is up-to-date
- [ ] Verify that new features are documented
- [ ] Bump version in `crate2nix/Cargo.toml`
- [ ] `./run_tests.sh` to regenerate sources after version bump
- [ ] Tag version e.g. `0.14.2` (without leading `v`)
- [ ] Push
- [ ] `cargo publish`
- [ ] In `flake.nix`: bump input `crate2nix_stable` to the new tag
- [ ] Create release from tag
