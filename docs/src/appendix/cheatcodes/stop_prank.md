# `stop_prank`

> `fn stop_prank(contract_address: ContractAddress)`

Cancels the [`start_prank`](./start_prank.md) for the contract at the given address.

- `contract_address` - target contract address

```rust
#[test]
fn test_prank() {
    // ...
    
    stop_prank(contract_address);
    
    // ...
}
```
