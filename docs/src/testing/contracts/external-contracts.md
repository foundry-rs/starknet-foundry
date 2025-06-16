# Using External Contracts in Tests

Contracts from external dependencies can be used in `snforge` tests.

## Add a Dependency

First, add a dependency on the contract package, either in `Scarb.toml` directly or by using `scarb add packageName`.
Read more about dependencies
in [Scarb documentation](https://docs.swmansion.com/scarb/docs/guides/dependencies.html#adding-a-dependency).

## Update `starknet-contract` Target

To use contract from dependencies in tests, `Scarb.toml` must be updated to include these contracts under
`[[target.starknet-contract]]`.

```toml
[[target.starknet-contract]]
build-external-contracts = ["externalPackage1::Contract1", "otherExternalPackage::path::to::Contract2"]
```

For more information about `build-external-contracts`,
see [Scarb documentation](https://docs.swmansion.com/scarb/docs/extensions/starknet/contract-target.html#compiling-external-contracts).
