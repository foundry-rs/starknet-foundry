# `stop_mock_call`

> `fn stop_mock_call(contract_address: ContractAddress, function_selector: felt252)`

Cancels the [`start_mock_call`](./start_mock_call.md) for the function `function_selector` of a contract at the given address.

- `contract_address` - target contract address
- `function_selector` - selector of the function

```rust
use snforge_std::stop_mock_call;

#[test]
fn test_mock_call() {
    // ...
    
    stop_mock_call(contract_address, selector!("my_function"));
    
    // ...
}
```
