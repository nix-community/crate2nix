# Release Checklist

[ ] `cargo test`
[ ] Verify build on Mac OS X
[ ] Verify that generated output looks nice
[ ] Verify that CHANGELOG.md is up-to-date
[ ] Verify that new features are documented in README.md
[ ] Bump version in Cargo.toml
[ ] Tag version
[ ] Publish release
[ ] Bump versions in README.md
[ ] Bump nixpkgs version with niv: niv update nixpkgs
[ ] Potentially check if tests work with stable nixpkgs