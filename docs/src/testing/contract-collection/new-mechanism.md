# How Contracts Are Collected

For the [`declare`](../../appendix/snforge-library/declare.md) to function, snforge must collect and call build on
contracts in the package. By default, if using Scarb version greater or equal to 2.8.3, snforge will combine test
collection and contract collection steps.

When running `snforge test`, snforge will, under the hood, call the `scarb build --test` command. This command builds
all the test and contracts along them. Snforge collects these contracts and makes them available for declaring in tests.

Contracts are collected from both `src` and `tests` directory, including modules marked with `#[cfg(test)]`.
Internally, snforge collects contracts from all `[[test]]` targets compiled by Scarb.
You can read more about that in [test collection](../test-collection.md) documentation.

## Collection Order

When multiple `[[test]]` targets are present, snforge will first try to collect contracts from `integration` `test-type`
target. If `integration` is not present, snforge will first collect contracts from the first encountered `[[test]]`
target.

After collecting from initial `[[test]]` target, snforge will collect contracts from any other encountered contracts.
No specific order of collection is guaranteed.

> 📝 **Note**
>
> If multiple contracts with the same name are present, snforge will use the first encountered implementation and will
> not collect others.

## Using External Contracts in Tests

To use contract from dependencies in tests `Scarb.toml` must be updated to include these contracts under
`[[target.starknet-contract]]`.

```toml
[[target.starknet-contract]]
build-external-contracts = ["path::to::Contract1", "other::path::to::Contract2"]
```
