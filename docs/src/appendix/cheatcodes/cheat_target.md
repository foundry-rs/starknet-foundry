# `CheatTarget`

```rust
enum CheatTarget {
    All: (),
    One: ContractAddress,
    Multiple: Array<ContractAddress>
}
```

`CheatTarget` is an enum used to designate the contracts to which a cheat should be applied. 
- `All` applies the cheatcode to all contract addresses. 
- `One` applies the cheatcode to the given contract address. 
- `Multiple` applies the cheatcode to the given contract addresses. 


An example with the [`start_warp`](./start_warp.md) cheatcode:
```rust 
#[test]
fn some_test() {
    // ...
    start_warp(CheatTarget::All, 123);
    // block timestamp for every address is set to 123
    start_warp(CheatTarget::One(10), 456);
    // block timestamp:
    //   - for address 10: 456
    //   - for every other address: 123
    stop_warp(CheatTarget::Multiple(array![20, 30]));
    // block timestamp:
    //   - for address 10: 456
    //   - for addresses 20 and 30: not cheated 
    //   - for every other address: 123
    stop_warp(CheatTarget::All);
    // block timestamp is not cheated for any address
    // ...
}
```
