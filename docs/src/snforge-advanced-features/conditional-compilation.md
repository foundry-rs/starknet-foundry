# Conditional Compilation

It is possible to build some contracts solely for testing purposes.
This can be achieved by leveraging [Scarb features](https://docs.swmansion.com/scarb/docs/reference/conditional-compilation.html#features).
Configuration in `Scarb.toml` is done in the same manner as described in the Scarb documentation.
Additionally, for utilizing features the `snforge test` command exposes the following flags, aligned with `scarb` flags:
[--features](../appendix/snforge/test.md#-f---features-features),
[--all-features](../appendix/snforge/test.md#--all-features) and [--no-default-features](../appendix/snforge/test.md#--no-default-features).

## Contracts

Firstly, define a contract in the `src` directory with a `#[cfg(feature: '<FEATURE_NAME>')]` attribute:

```rust
#[starknet::interface]
trait IMockContract<TContractState> {
    fn response(self: @TContractState) -> u32;
}

#[starknet::contract]
#[cfg(feature: 'enable_for_tests')]
mod MockContract {
    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl IContractImpl of super::IContract<ContractState> {
        fn response(self: @ContractState) -> u32 {
            1
        }
    }
}
```

> 📝 **Note**
> To declare mock contracts in tests, these contracts should be defined within the package and not in the `tests` directory.
> This requirement is due to the way snforge [collects contracts](../testing/contracts-collection.md).


Next, create a test that uses the above contract:

```rust
#[test]
fn test_mock_contract() {
    let (contract_address, _) = declare("MockContract")
        .unwrap()
        .contract_class()
        .deploy(@array![])
        .unwrap();
    let response_result = IContractDispatcher { contract_address }.response();
    assert(response_result == 1, '');
}
```

The `Scarb.toml` file needs to be updated so it includes the following lines:

```toml
[features]
enable_for_tests = []
```

Then, to use the contract in tests `snforge test` must be provided with a flag defined above:

```
snforge test --features enable_for_tests
```

> 📝 **Note**
> If `snforge test` is run without the above feature enabled, it won't build any artifacts for the `MockContract` and all tests that use this contract will fail.

## Functions

Features are not limited to conditionally compiling contracts and can be used with other parts of the code, like functions:

```rust
#[cfg(feature: 'enable_for_tests')]
fn mock_function() -> u32 {
    2
}

#[test]
fn test_using_mock_function() {
    assert!(mock_function() == 2, '');
}
```
