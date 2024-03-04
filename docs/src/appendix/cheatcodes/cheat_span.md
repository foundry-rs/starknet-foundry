# `CheatSpan`

```rust
enum CheatSpan {
    Indefinite: (),
    Calls: usize
}
```

`CheatSpan` is an enum used to specify for how long the target should be cheated.
- `Indefinite` applies the cheatcode for indefinitely, until the cheat is stopped manually (e.g. using `stop_warp`).
- `Calls` applies the cheatcode for specified number of calls, after which the cheat is stopped. 


An example with the [`start_warp`](./start_warp.md) cheatcode:
```rust 
#[test]
fn some_test() {
    // ...

    warp(CheatTarget::One(contract_address), 123, CheatSpan::Indefinite);
    // block timestamp for contract_address is set to 123 until stop_warp is called
    
    // ...
}
```

```rust 
#[test]
fn other_test() {
    // ...
    // assume get_block_timestamp() simply returns the timestamp

    warp(CheatTarget::One(contract_address), 123, CheatSpan::Calls(2));
    // block timestamp for contract_address is set to 123 for the next 2 calls
    // or until stop_warp is called

    let timestamp = dispatcher.get_block_timestamp()
    assert_eq!(timestamp, 123);

    // block timestamp for contract_address is set to 123 for the next 1 call
    let timestamp = dispatcher.get_block_timestamp()
    assert_eq!(timestamp, 123);

    // block timestamp is not cheated anymore for contract_address
    // ...
}
```
