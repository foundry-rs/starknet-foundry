# `stop_warp`

> `fn stop_warp(target: CheatTarget)`

Cancels the [`start_warp`](./start_warp.md) for the given target.

- `target` - instance of [`CheatTarget`](./cheat_target.md) specifying which contracts to stop warping.

```rust
use snforge_std::stop_warp;

#[test]
fn test_warp() {
    // ...
    
    stop_warp(CheatTarget::One(contract_address));
    
    // ...
}
```
