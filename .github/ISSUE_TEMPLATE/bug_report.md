---
name: Bug Report
about: Report a bug with crate2nix
title: ''
labels: bug
assignees: ''
---

## Description

A clear description of the bug.

## Steps to Reproduce

1.
2.
3.

## Expected Behavior

What you expected to happen.

## Actual Behavior

What actually happened. Include error messages if applicable.

## Environment

- crate2nix version:
- Nix version:
- OS:

## Reproducible Test Case

**To help us fix this issue quickly, please provide a minimal reproducible example.**

The best way to do this is to submit a PR that adds a sample project to `sample_projects/`:

1. Create a minimal Cargo project that reproduces the issue
2. Add it to `sample_projects/your_project_name/`
3. Run `./regenerate_cargo_nix.sh` to generate the `Cargo.nix`
4. The test suite (`./run_tests.sh` or `nix build -L -f ./tests.nix`) should demonstrate the failure

See existing projects in `sample_projects/` for examples of the expected structure.

If you can't submit a PR, please share:

- Your `Cargo.toml` and `Cargo.lock`
- Any relevant `crate2nix.json` configuration
- The generated `Cargo.nix` (or the error if generation fails)
