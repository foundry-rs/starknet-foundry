# `stop_spoof`

> `fn stop_spoof(target: CheatTarget)`

Cancels the [`start_spoof`](./start_spoof.md) for the given target.

- `target` - instance of [`CheatTarget`](./cheat_target.md) specifying which contracts to stop spoofing

```rust
use snforge_std::{ stop_spoof, CheatTarget };

#[test]
fn test_spoof() {
    // ...
    
    stop_spoof(CheatTarget::One(contract_address));
    
    // ...
}
```
