# `stop_elect`

> `fn stop_elect(target: CheatTarget)`

Cancels the [`start_elect`](./start_elect.md) for the given target.

- `target` - instance of [`CheatTarget`](./cheat_target.md) specifying which contracts to stop electing

```rust
use snforge_std::{stop_elect, CheatTarget};

#[test]
fn test_elect() {
    // ...
    
    stop_elect(CheatTarget::One(contract_address));
    
    // ...
}
```
