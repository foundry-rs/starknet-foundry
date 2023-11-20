# `stop_roll`

> `fn stop_roll(target: CheatTarget)`

Cancels the [`start_roll`](./start_roll.md) for the given target.

- `target` - instance of [`CheatTarget`](./cheat_target.md) specifying which contracts to stop rolling

```rust
use snforge_std::{stop_roll, CheatTarget};

#[test]
fn test_roll() {
    // ...
    
    stop_roll(CheatTarget::One(contract_address));
    
    // ...
}
```
