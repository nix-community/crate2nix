
# Handling out-of-tree sources

NOTE: This is work in progress, see
[#102](https://github.com/kolloch/crate2nix/issues/102).

`crate2nix` has convenient support for managing out-of-tree sources: It will
manage a nix-generated [Cargo
Workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) for
you.

Cargo will resolve dependency versions across all the binaries/libs in this
workspace. Given lax enough version requirements, that makes it more likely that
the dependency trees of your binaries/libraries will overlap.

## Demo

Starting from scratch with an empty directory:

```
❯ mkdir ripgrep-example

❯ cd ripgrep-example
```

Adding a crates.io dependency by name and version:

```
❯ crate2nix source add cratesIo ripgrep 12.0.1
Prefetching https://crates.io/api/v1/crates/ripgrep/12.0.1/download: done.
Added new source: ripgrep 12.0.1 from crates.io: 1arw9pk1qiih0szd26wq76bc0wwbcmhyyy3d4dnwcflka8kfkikx
```

Adding a git dependency by URL and revision hash:

```
❯ crate2nix source add git https://github.com/cjbassi/ytop --rev 21cb63f656519b86928fce9522107f78780d6460
Prefetching https://github.com/cjbassi/ytop#21cb63f656519b86928fce9522107f78780d6460: done.
Added new source: https://github.com/cjbassi/ytop#21cb63f656519b86928fce9522107f78780d6460 via git: 1si5cq18qn819v28vfgbrx3fmfgzj4h0z00ga2p0px42q62vrs7q
```

Listing all our sources:

```
❯ crate2nix source list
ripgrep ripgrep 12.0.1 from crates.io: 1arw9pk1qiih0szd26wq76bc0wwbcmhyyy3d4dnwcflka8kfkikx

        crate2nix source add cratesIo --name 'ripgrep' 'ripgrep' '12.0.1'

ytop    https://github.com/cjbassi/ytop#21cb63f656519b86928fce9522107f78780d6460 via git: 1si5cq18qn819v28vfgbrx3fmfgzj4h0z00ga2p0px42q62vrs7q

        crate2nix source add git --name 'ytop' 'https://github.com/cjbassi/ytop' --rev 21cb63f656519b86928fce9522107f78780d6460

```

This also shows the commands with which you could recreate your sources for
convenience.

Generating `Cargo.nix`, prefetching some indirect git dependencies:

```
❯ crate2nix generate
Generated ./workspace.nix successfully.
Building ./workspace.nix workspaceMemberDirectory: done.
Generated ./Cargo.toml successfully.
Updating Cargo.lock: done.
Prefetching    1/2: https://github.com/rust-psutil/rust-psutil#e295e4f815942a5700d7bb11de2396906a8116bc
Prefetching    2/2: https://github.com/cjbassi/tui-rs#aff0a4c40aff6e0962a2a935d4b21065298e329c
Wrote hashes to ./crate-hashes.json.
Generated ./Cargo.nix successfully.
```

Building the binaries:

```
❯ nix build -f Cargo.nix workspaceMembers.ripgrep workspaceMembers.ytop
[93 built, 0.0 MiB DL]
```

Running the binaries:

```
❯ ./result-1/bin/ytop --version
ytop 0.5.1
❯ ./result/bin/rg --version
ripgrep 12.0.1
-SIMD -AVX (compiled)
+SIMD +AVX (runtime)
```

## Managing sources in crate2nix.json

Here are some examples for adding sources:

```
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

* Build a workspace directory with all your sources as members.
  It uses a generated `workspace.nix` in your project directory.
* Generate a matching `Cargo.toml`.
* Call`cargo generate-lockfile` to create a `Cargo.lock` file.

## crate2nix cargo-files update

`crate2nix cargo-files update` is what `crate2nix generate` calls under the hood
to create a directory with all workspace members and to generate the appropriate
`Cargo.toml`/`Cargo.lock` for your nix-generated workspace.

If you do not use `crate2nix generate`, you can call this directly to prepare
the sources for `appliedCargoNix` (see `tools.nix`) and friends.

## What to check into version control

For building the binaries, `Cargo.nix` is sufficient. But everyone regenerating
it will appreciate if you also check in

* crate2nix.json (the source configuration)
* crate-hashes.json (the hashes for packages not in the lockfile)
* Cargo.lock (required if you use import from derivation, otherwise it will
  probably also save you time)

## Feature resolution

Note that features in `crate2nix` are resolved at build time so that every
dependency is build only with the features necessary for the specific binary.
This is probably what you want because it can prevent build problems. Even
though, features in rust are meant to be additive, in reality they are often
not.
