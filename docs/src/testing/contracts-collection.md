### How Contracts Are Collected

When you call `snforge test`, one of the things that Forge does is that it calls Scarb, particularly `scarb build`.
It makes Scarb build all contracts from your package and save them to the `target/{current_profile}` directory
(read more on [Scarb website](https://docs.swmansion.com/scarb/docs/extensions/starknet/contract-target.html)).

Then, Foundry loads compiled contracts from each package (for all packages in the project) and uses them to run tests.

> ⚠️ **Warning**
>
> Make sure to define `[[target.starknet-contract]]` section in your `Scarb.toml`, otherwise Scarb won't build your contracts.


## Using External Contracts In Tests

If you wish to use external contracts inside your tests (e.g. an ERC20 token, an account contract) from your dependencies,
you must first make Scarb build them.  You can do that by using `build-external-contracts` property in `Scarb.toml`, e.g.: 

```toml
[[target.starknet-contract]]
build-external-contracts = ["openzeppelin::account::account::Account"]
```

For more information about `build-external-contracts`, see [Scarb documentation](https://docs.swmansion.com/scarb/docs/extensions/starknet/contract-target.html#compiling-external-contracts).
