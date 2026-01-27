---
title: Generating Cargo.nix for multiple fetched sources
sidebar:
  badge: { text: 'Experimental', variant: 'caution' }
---

NOTE: This is work in progress, see
[#102](https://github.com/kolloch/crate2nix/issues/102), the interface might
still change.

`crate2nix` has convenient support for managing out-of-tree sources: It will
manage a nix-generated directory of the source roots for you.

`crate2nix` will use the supplied `Cargo.lock` files in the sources to generate
the binaries with the versions that the maintainers specified.

## Demo

Starting from scratch with an empty directory:

```console
❯ mkdir ripgrep-example

❯ cd ripgrep-example
```

Adding a crates.io dependency by name and version:

```console
❯ crate2nix source add cratesIo ripgrep 12.0.1
Prefetching https://crates.io/api/v1/crates/ripgrep/12.0.1/download: done.
Added new source: ripgrep 12.0.1 from crates.io: 1arw9pk1qiih0szd26wq76bc0wwbcmhyyy3d4dnwcflka8kfkikx
```

Adding a git dependency by URL and revision hash:

```console
❯ crate2nix source add git https://github.com/cjbassi/ytop --rev 21cb63f656519b86928fce9522107f78780d6460
Prefetching https://github.com/cjbassi/ytop#21cb63f656519b86928fce9522107f78780d6460: done.
Added new source: https://github.com/cjbassi/ytop#21cb63f656519b86928fce9522107f78780d6460 via git: 1si5cq18qn819v28vfgbrx3fmfgzj4h0z00ga2p0px42q62vrs7q
```

Listing all our sources:

```console
❯ crate2nix source list
ripgrep ripgrep 12.0.1 from crates.io: 1arw9pk1qiih0szd26wq76bc0wwbcmhyyy3d4dnwcflka8kfkikx

        crate2nix source add cratesIo --name 'ripgrep' 'ripgrep' '12.0.1'

ytop    https://github.com/cjbassi/ytop#21cb63f656519b86928fce9522107f78780d6460 via git: 1si5cq18qn819v28vfgbrx3fmfgzj4h0z00ga2p0px42q62vrs7q

        crate2nix source add git --name 'ytop' 'https://github.com/cjbassi/ytop' --rev 21cb63f656519b86928fce9522107f78780d6460

```

This also shows the commands with which you could recreate your sources for
convenience.

Generating `Cargo.nix`, prefetching some indirect git dependencies:

```console
❯ crate2nix generate
Fetching sources.
Generated ./crate2nix-sources.nix successfully.
Fetching sources via ./crate2nix-sources.nix fetchedSources: done.
Prefetching    1/2: https://github.com/rust-psutil/rust-psutil#6abe2de4409672c3f42b69db2b1ba45d73e78ee4
Prefetching    2/2: https://github.com/cjbassi/tui-rs#aff0a4c40aff6e0962a2a935d4b21065298e329c
Wrote hashes to ./crate-hashes.json.
Generated ./Cargo.nix successfully.
```

Building the binaries:

```console
❯ nix build -f Cargo.nix workspaceMembers.ripgrep workspaceMembers.ytop
[93 built, 0.0 MiB DL]
```

Running the binaries:

```console
❯ ./result-1/bin/ytop --version
ytop 0.5.1
❯ ./result/bin/rg --version
ripgrep 12.0.1
-SIMD -AVX (compiled)
+SIMD +AVX (runtime)
```

## Managing sources in crate2nix.json

Here are some examples for adding sources:

```console
crate2nix source add cratesIo ripgrep 12.0.1
crate2nix source add git https://github.com/kolloch/crate2nix.git --rev 0832e5ac0a2c53a7a99b9b0b2ff2d51828e5cb60
crate2nix source add nix --import ./nix/sources.nix my_crate
```

If `crate2nix.json` does not exist yet, it will be created.

As always, you can have a look at all available options and commands by using
`crate2nix source --help`, `crate2nix source add --help` and so forth.

`crate2nix source list` shows already configured sources.

`crate2nix source remove ripgrep` removes the source named `ripgrep`.

## crate2nix generate

`crate2nix generate` will generally just do the right thing and pick up
crate2nix.json file if necessary:

* Build a `crate-sources` directory with all of your sources.
  It uses a generated `crate-sources.nix` in your project directory.
* Call `cargo metadata` individually for all sources and concatenate the
  results.

## What to check into version control

For building the binaries, `Cargo.nix` is sufficient. But everyone regenerating
it will appreciate if you also check in

* `crate2nix.json` (the source configuration)
* `crate-hashes.json` (the hashes for packages not in the lockfile)

The generated `crate-sources*` files should go into `.gitignore` or similar:

```crate-sources*```

* `crate-sources.nix` is the temporary nix expression to download the sources.
* `crate-sources` is the temporary result symlink to the downloaded sources.

## Feature resolution

Note that features in `crate2nix` are resolved at build time so that every
dependency is build only with the features necessary for the specific binary.
This is probably what you want because it can prevent build problems. Even
though, features in rust are meant to be additive, in reality they are often
not.
