# `stop_warp`

> `fn stop_warp(contract_address: ContractAddress)`

Cancels the [`start_warp`](./start_warp.md) for the contract at the given address.

- `contract_address` - target contract address

```rust
use cheatcodes::stop_warp;

#[test]
fn test_wrap() {
    // ...
    
    stop_warp(contract_address);
    
    // ...
}
```
