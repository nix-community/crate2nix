---
title: Building binaries
---

## Single binary crates

If your `Cargo.nix` was generated for a single binary crate (i.e. workspace)
then the derivation that builds your binary can be accessed via the
`rootCrate.build` attribute. Use this command to build it and make the result
available in the result directory:

```bash
your_crate_name="super_duper"
nix build -f Cargo.nix rootCrate.build
./result/bin/${your_crate_name}
```

Within a nix file (e.g. your manually written `default.nix`), you can access the
derivation like this:

```nix
let cargo_nix = callPackage ./Cargo.nix {};
in cargo_nix.rootCrate.build
```

## Cargo workspaces with multiple crates

If your `Cargo.nix` was generated for a workspace (i.e. not a single binary)
then the derivation that builds your binary CANNOT be accessed via the
`rootCrate` attribute. There is no single root crate.

Instead, you can conveniently access the derivations of all your workspace
members through the `workspaceMembers` attribute. Use this command to build one
of the workspace members and make the result available in the result directory:

```bash
your_crate_name="super_duper"
nix build -f Cargo.nix workspaceMembers.${your_crate_name}.build
./result/bin/${your_crate_name}
```

Within a nix file (e.g. your manually written `default.nix`), you can access the
derivation like this:

```nix
let cargo_nix = callPackage ./Cargo.nix {};
in cargo_nix.workspaceMembers."${your_crate_name}".build
```

### Evaluating many workspace members at once

Each `workspaceMembers.<name>.build` instantiates the derivations of its own
dependency closure, so evaluating all members of a large workspace instantiates
dependencies they share once per member. If you need all members anyway (for
example to build or check the whole workspace), use `workspaceMembersShared`
instead:

```nix
let cargo_nix = callPackage ./Cargo.nix {};
in cargo_nix.workspaceMembersShared."${your_crate_name}"
```

Each member's derivation is identical to its `workspaceMembers.<name>.build`,
but all members are instantiated over one shared table of crate derivations.
This makes evaluating many members that share dependencies much cheaper.
The trade-offs:

* Forcing any one member evaluates all of them, so for a single member or a
  small workspace, `workspaceMembers.<name>.build` is cheaper.
* All members use `rootFeatures`; there is no per-member `.override`.
