# `stop_warp_global`

> `fn stop_warp_global()`

Cancels [`start_warp_global`](./start_warp.md). 

```rust
use snforge_std::stop_warp;

#[test]
fn test_warp() {
    // ...
    
    stop_warp_global();
    
    // ...
}
```
