# How Contracts Are Collected

When you call `snforge test`, one of the things that `snforge` does is that it calls Scarb, particularly `scarb build`.
It makes Scarb build all contracts from your package and save them to the `target/{current_profile}` directory
(read more on [Scarb website](https://docs.swmansion.com/scarb/docs/extensions/starknet/contract-target.html)).

Then, `snforge` loads compiled contracts from the package your tests are in, allowing you to declare the contracts in
tests.

Only contracts from `src/` directory are loaded. Contracts from `/tests` and modules marked with `#[cfg(test)]` are not
build or collected. To create contracts to be specifically used in tests
see [conditional compilation](../../snforge-advanced-features/conditional-compilation.md).

> ⚠️ **Warning**
>
> Make sure to define `[[target.starknet-contract]]` section in your `Scarb.toml`, otherwise Scarb won't build your
> contracts.

## Using External Contracts In Tests

If you wish to use contracts from your dependencies inside your tests (e.g. an ERC20 token, an account contract),
you must first make Scarb build them. You can do that by using `build-external-contracts` property in `Scarb.toml`,
e.g.:

```toml
[[target.starknet-contract]]
build-external-contracts = ["openzeppelin::account::account::Account"]
```

For more information about `build-external-contracts`,
see [Scarb documentation](https://docs.swmansion.com/scarb/docs/extensions/starknet/contract-target.html#compiling-external-contracts).
