# `stop_mock_call`

> `fn stop_mock_call(contract_address: ContractAddress, function_name: felt252)`

Cancels the [`start_mock_call`](./start_mock_call.md) for the function `function_name` of a contract at the given address.

- `contract_address` - target contract address
- `function_name` - name of the function

```rust
use snforge_std::stop_mock_call;

#[test]
fn test_mock_call() {
    // ...
    
    stop_mock_call(contract_address, function_name);
    
    // ...
}
```
