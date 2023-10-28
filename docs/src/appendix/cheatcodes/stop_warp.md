# `stop_warp`

> `fn stop_warp(target: CheatTarget)`

Cancels the [`start_warp`](./start_warp.md) for the contract at the given address(es).

- `target` - target contract address(es)

```rust
use snforge_std::stop_warp;

#[test]
fn test_warp() {
    // ...
    
    stop_warp(CheatTarget::One(contract_address));
    
    // ...
}

#[test]
fn test_warp2() {
    // ...
    
    stop_warp(CheatTarget::Multiple(array![address1, address2, address3].span()));
    
    // ...
}

#[test]
fn test_warp3() {
    // ...

    stop_warp(CheatTarget::All);
    
    // ...
}


```
