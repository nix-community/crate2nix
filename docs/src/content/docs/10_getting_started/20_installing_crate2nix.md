---
title: Installing crate2nix
---

Know what you are doing and want a stable version?
Just use the version from [nixpkgs](https://search.nixos.org/packages?channel=unstable&show=crate2nix&from=0&size=50&sort=relevance&type=packages&query=crate2nix).

:::note

New to nix? [Install nix first](https://github.com/DeterminateSystems/nix-installer).

:::

## Flake-enabled nix

```bash title="Install from nixpkgs in user profile (recommended)"
nix profile install nixpkgs#crate2nix
crate2nix help
```

```bash title="Install latest development version"
nix profile install github:nix-community/crate2nix
crate2nix help
```

## Traditional (non-flake)

```bash title="Install from nixpkgs (recommended)"
nix-channel --update # if you wish
nix-env -i -f '<nixpkgs>' -A crate2nix
```

```bash title="Install latest development version"
nix-env -i -f https://github.com/nix-community/crate2nix/tarball/master
```
