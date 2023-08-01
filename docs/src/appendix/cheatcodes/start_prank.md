# `start_prank`

> `fn start_prank(contract_address: ContractAddress, caller_address: ContractAddress)`

Changes the caller address in the `ExecutionInfo` struct and methods utilizing it (e.g. `get_caller_address`).

- `contract_address` - address of the contract to be pranked
- `caller_address` - caller address to be set

```rust
#[test]
fn test_prank_simple() {
    // ...
    
    let caller_address: felt252 = 123;
    let caller_address: ContractAddress = caller_address.try_into().unwrap();

    start_prank(contract_address, caller_address);

    let caller_address = dispatcher.get_caller_address();
    assert(caller_address == 123, 'Wrong caller address');
}
```