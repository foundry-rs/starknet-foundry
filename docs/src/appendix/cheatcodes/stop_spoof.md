# `stop_spoof`

> `fn stop_spoof(contract_address: ContractAddress)`

Cancels the [`start_spoof`](./start_spoof.md) for the contract at the given address.

- `contract_address` - target contract address

```rust
use snforge_std::cheatcodes::stop_spoof;

#[test]
fn test_spoof() {
    // ...
    
    stop_spoof(contract_address);
    
    // ...
}
```
