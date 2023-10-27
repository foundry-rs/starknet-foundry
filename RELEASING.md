# Instruction For Creating New Starknet Forge Releases

1. Create a new branch.
2. Run `./scripts/release.sh MAJOR.MINOR.PATCH`.
3. Merge introduced changes to master branch.
4. Wait for release workflows to pass. A new release will be created on GitHub.

## Manually Creating a Release

In case a manual creation of release is necessary, for example when
cherry-picking changes, a release can also be triggered by creating a tag
with the name format `vMAJOR.MINOR.PATCH`.

Note that in this case `CHANGELOG.md`, `Cargo.toml` and `Cargo.lock` files
have to be updated accordingly.
