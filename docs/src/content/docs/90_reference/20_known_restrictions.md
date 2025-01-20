---
title: Known Restrictions
---

`crate2nix` makes heavy use of `buildRustCrate` in `nixpkgs`. So we potentially
depend on features in a recent version of `nixpkgs`. Check
[nix/sources.json](https://github.com/nix-community/crate2nix/blob/master/nix/sources.json)
for the version of nixpkgs that `crate2nix` is tested against.

If you feel limited by these restrictions, please do not hesitate to file an issue! That
gives me a feeling of what is worth working on.

* There is only experimental support for running tests ~~Before 0.7.x: No
  support for building and running tests, see [nixpkgs, issue
  59177](https://github.com/NixOS/nixpkgs/issues/59177).~~
* Target-specific features do not work automatically, see
  [#129](https://github.com/nix-community/crate2nix/issues/129). You should be able to
  enable the required features manually, however.
* A crate will only have access to its own source directory during build time
  and not e.g. to other directories in the same workspace. See [crate2nix, issue
  17](https://github.com/nix-community/crate2nix/issues/17). I used to consider this
  "works as intended" but now I think that we should use the "workspaceMember"
  argument of buildRustCrate for this.
* It does translates target strings to nix expressions. The support should be
  reasonable but probably not complete - please let me know if you hit problems.
  ~~Before 0.2.x: Filters all dependencies for the *hard-coded "Linux x86_64"
  target platform*. Again, it should be quite easy to support more platforms. To
  do so completely and at build time (vs build generation time) might be more
  involved.~~

Former restrictions, now supported:

* ~~Before 0.8.x: Since cargo exposes local paths in package IDs, the generated
  build file also contain them as part of an "opaque" ID. They are not
  interpreted as paths but maybe you do not want to expose local paths in
  there...~~ The full opaque package ID will only be used if you have the same
  package with the same version multiple times. That should be very rare.
* ~~Before 0.6.x: [Renamed
  crates](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#renaming-dependencies-in-cargotoml)
  with an explicit `package` name don't work yet.~~
* Git sources are now also supported. Starting with 0.7 sub modules also work.
  Finding crates in arbitrary sub directories of git sources (which cargo
  supports!)is not supported, see #53.
* ~~Before 0.4.x: Only *default crate features* are supported. It should be easy
  to support a different feature set at build generation time since we can
  simply pass this set to `cargo metadata`. Feature selection during build time
  is out of scope for now.~~
