# Instruction For Creating New Starknet Forge Releases

1. Bump Starknet Foundry version in the top-level `Cargo.toml` file
2. Regenerate locks using `cargo update -p forge cast`
3. Update `CHANGELOG.md`
3. Merge introduced changes
4. Create a new tag in repository with format `vMAJOR.MINOR.PATCH`. This will trigger the release workflow
5. Wait for release workflows to pass. A new draft release will be created on GitHub.
6. Update the release contents using template below and publish it

## Release Template

Optional release description

## What's Changed

- list
- of
- features

## Compatible Scarb Versions

| Tool      | Scarb Version |
|-----------|---------------|
| `snforge` | `X.Y.Z`       |
| `sncast`  | `X.Y.Z`       |
