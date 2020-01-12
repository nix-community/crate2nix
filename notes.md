# Release Checklist

- [ ] Bump nixpkgs version with niv: niv update nixpkgs
- [ ] `./run_tests.sh`
- [ ] Verify build on Mac OS X
- [ ] Verify that generated output looks nice
- [ ] Verify that CHANGELOG.md is up-to-date
- [ ] Verify that new features are documented in README.md
- [ ] Bump version in Cargo.toml: `cargo release Major/Minor/Alpha`
- [ ] Bump versions in README.md
