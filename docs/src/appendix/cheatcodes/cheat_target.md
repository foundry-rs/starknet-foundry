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
- `Multiple` applies the cheatcode to each given address. 

> 📝 **Note**
> 
> `CheatTarget::Multiple` acts as a helper for targeting every specified address separately with `CheatTarget::One`.
