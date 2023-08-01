# `start_prank`

> `fn start_prank(contract_address: ContractAddress, caller_address: ContractAddress)`

Changes the caller address for a contract at the given address.
This change can be canceled with [`stop_prank](./stop_prank.md).

- `contract_address` - target contract address
- `caller_address` - caller address to be set

For contract implementation:

```rust
// ...
#[external(v0)]
impl IContractImpl of IContract<ContractState> {
    fn get_caller_address( ref self: ContractState) -> ContractAddress {
        starknet::get_caller_address()
    }
}
// ...
```

We can use `start_prank` in a test to change the caller address for a given contract:

```rust
#[test]
fn test_prank_simple() {
    // ...

    let caller_address: ContractAddress = 123.try_into().unwrap();

    start_prank(contract_address, caller_address);

    let caller_address = dispatcher.get_caller_address();
    assert(caller_address.into() == 123, 'Wrong caller address');
}
```