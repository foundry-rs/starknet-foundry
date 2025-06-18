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

# Nightly releases

Nightly release must be triggered manually using the Nightly GitHub action.
This action builds binaries from specified ref and uploads them to the [starknet-foundry-nightlies](https://github.com/software-mansion-labs/starknet-foundry-nightlies) repository.
Additionally, there are `stds` and `plugin` versions uploaded to the [_dev_ registry](https://scarbs.dev/).
After a successful release, [Ma'at](https://github.com/software-mansion/maat) is automatically triggered to run experiments in nightly workspace. Results can be found [here](https://docs.swmansion.com/maat/)

### Maintenance

Some access tokens require annual renewal due to expiration:
- `SNFOUNDRY_NIGHTLIES_CONTENTS_WRITE` - grants permission to create releases in nightlies repository.
- `MAAT_CONTENTS_READ_ACTIONS_WRITE` - required to trigger Ma'at run.
