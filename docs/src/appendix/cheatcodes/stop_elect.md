# `stop_elect`

> `fn stop_elect(contract_address: ContractAddress)`

Cancels the [`start_elect`](./start_elect.md) for the contract at the given address.

- `contract_address` - target contract address

```rust
use snforge_std::stop_elect;

#[test]
fn test_elect() {
    // ...
    
    stop_elect(contract_address);
    
    // ...
}
```
