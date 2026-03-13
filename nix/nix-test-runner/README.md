# nix-test-runner

This is a mostly-vendored version of the nix source from
[NixTest](https://github.com/stoeffel/nix-test-runner), mainly to work around the `lib ? (import
<nixpkgs> { }).lib` argument in the upstream `runTests.nix`.
