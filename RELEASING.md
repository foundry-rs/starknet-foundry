# Instruction For Creating New Starknet Forge Releases

1. Bump Starknet Forge versions in
    1. Top-level `Cargo.toml`
    2. `crates/forge/Cargo.toml`
    3. `crates/cast/Cargo.toml`
    4. Any other applicable user-facing tools
2. Regenerate locks using `cargo generate-lockfile`
3. Merge introduced changes
4. Create a new tag in repository with format `vMAJOR.MINOR.PATCH`
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
