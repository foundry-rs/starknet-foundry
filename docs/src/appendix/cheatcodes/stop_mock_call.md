# `stop_mock_call`

> `fn stop_mock_call(contract_address: ContractAddress, fn_name: felt252)`

Cancels the [`start_mock_call`](./start_mock_call.md) for the function `fn_name` in the contract at the given address.

- `contract_address` - target contract address
- `fn_name` - name of the function

```rust
#[test]
fn test_prank() {
    // ...
    
    stop_mock_call(contract_address, fn_name);
    
    // ...
}
```
