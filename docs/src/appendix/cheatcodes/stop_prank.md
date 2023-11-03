# `stop_prank`

> `fn stop_prank(target: CheatTarget)`

Cancels the [`start_prank`](./start_prank.md) for the given target.

- `contract_address` - target contract address

```rust
use snforge_std::{stop_prank, CheatTarget};

#[test]
fn test_prank() {
    // ...
    
    stop_prank(CheatTarget::One(contract_address));
    
    // ...
}
```
