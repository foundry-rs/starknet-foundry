# Snapshot tests
> 💡 **Info**
> This tutorial is only relevant if you wish to contribute to Starknet Foundry. 
> If you plan to only use it as a tool for your project, you can skip this part.

Some Forge tests use [insta](https://insta.rs/) to store expected test output. 
This allows us to test behavior across Scarb versions supported by Starknet Foundry.

## Prefix

All snapshot tests **must be** prefixed with `snap_` prefix.

As of writing of this document, these are the tests that use `assert_cleaned_output!`.
This allows us to explicitly run these on CI.

## The check script

Locally, **`scripts/check_snapshot.sh`** script can be used to run snapshot tests (and fix snapshots).

### Usage

To make sure snapshot tests pass for all currently supported Scarb versions, run:
```sh
./scripts/check_snapshots.sh
```
If some of the snapshot tests fail, run:
```sh
./scripts/check_snapshots.sh --fix
```
and review the newly generated snapshots.
