# Conditional Compilation

It is possible to build some contracts solely for testing purposes.
This can be achieved by leveraging [Scarb features](../snforge-advanced-features/fork-testing.md).
Configuration in `Scarb.toml` is done in the same manner as described in the Scarb documentation.
Additionally, the `snforge test` command exposes the following flags: [--features](../appendix/snforge/test.md#-f---features-features),
[--all-features](../appendix/snforge/test.md#--all-features) and [--no-default-features](../appendix/snforge/test.md#--no-default-features).

## Contracts

Firstly, define a contract in the `src` directory with a `#cfg(feature: (...))` attribute:

```rust
#[starknet::interface]
trait IMockContract<TContractState> {
    fn response(self: @TContractState) -> u32;
}

#[starknet::contract]
#[cfg(feature: 'snforge_test_only')]
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

Next, create a test with the same attribute:

```rust
#[test]
#[cfg(feature: 'snforge_test_only')]
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

The `Scarb.toml` file needs to be updated:

```toml
[features]
snforge_test_only = []
```

Then, tests can be executed with:

```
snforge test --features snforge_test_only
```

If `snforge test` is run without features enabled, it won't execute the previous test or build any artifacts for the `MockContract`.

## Functions

Similarly, we can conditionally compile some functions created in the `tests` directory:

```rust
#[cfg(feature: 'snforge_test_only')]
fn mock_function() -> u32 {
    2
}

#[test]
#[cfg(feature: 'snforge_test_only')]
fn test_using_mock_function() {
    assert!(mock_function() == 2, '');
}
```
