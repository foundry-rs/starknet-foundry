# 'CheatTarget`

```rust
enum CheatTarget {
    All: (),
    One: ContractAddress,
    Multiple: Span<ContractAddress>
}
```

`CheatTarget` is an enum used to designate the contracts to which a warp should be applied. 
- `All` applies the cheatcode to all contract addresses. 
- `One` applies the cheatcode to the given contract address. 
- `Multiple` applies the cheatcode to the given contract addresses. 


```rust 
#[test]
fn some_test() {
    // ...
    start_warp(CheatTarget::All, 123);
    // ...
}
```