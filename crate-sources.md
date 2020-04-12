This feature will make use of a newly introduced crate2nix.json file.

# Adding a new source to crate2nix.json

```bash
crate2nix source add cratesio ripgrep
crate2nix source add ripgrep-latest cratesio ripgrep
crate2nix source add cratesio ripgrep 10.2.0
crate2nix source add cratesio ripgrep 10.2.0 "<hash>"
crate2nix source add git https://github.com/kolloch/crate2nix.git
crate2nix source add git https://github.com/kolloch/crate2nix.git -b master
crate2nix source add git https://github.com/kolloch/crate2nix.git
crate2nix source add github kolloch/crate2nix
```

* If `crate2nix.json` does not exist yet, it will be created.
* If the version is omitted, the most recent is used.
* If the hash is omitted, the source is prefetched and it is calculated.

`crate2nix source modify ...`: Change an existing source
`crate2nix source modify -a ...`: Update an existing source or add it

`crate2nix source drop ripgrep`
`crate2nix source show ripgrep`

`crate2nix source update`: Update all

# crate2nix generate

will pick up crate2nix.json and if necessary:

* Builds a workspace
  * Generate derivation nix in
* Builds a cargo toml
* calls `cargo generate-lockfile` or unfreezes lock file

Then do the generate:

* Workspace sources will be taken from crate2nix.json ala
  `src = crate2nixJsonSources."ripgrep";`.
