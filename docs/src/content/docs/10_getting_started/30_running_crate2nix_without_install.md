---
title: Running crate2nix without installation
---

Know what you are doing and want a stable version?
Just use the version from [nixpkgs](https://search.nixos.org/packages?channel=unstable&show=crate2nix&from=0&size=50&sort=relevance&type=packages&query=crate2nix).

:::note

New to nix? [Install nix first](https://github.com/DeterminateSystems/nix-installer).

:::

## Flake-enabled nix

```bash title="Running from nixpkgs without installation"
nix run nixpkgs#crate2nix -- help
```

```bash title="Running latest development version without installation"
nix run github:nix-community/crate2nix -- help
```
