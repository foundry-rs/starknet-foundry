# `declare`

> `fn declare(contract: felt252) -> ContractClass`

Declares a contract and returns `ContractClass`.

- `contract` - name of a contract as Cairo shortstring. It is a name of the contract (part after `mod` keyword) e.g. `'HelloStarknet'`

```rust
use result::ResultTrait;

#[test]
fn test_declare() {
    let contract = declare('HelloStarknet');
    // ...
}
```
