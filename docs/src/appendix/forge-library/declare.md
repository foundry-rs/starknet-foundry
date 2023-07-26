# `declare`

> `fn declare(contract: felt252) -> felt252`

Declares a contract and returns its class hash.

- `contract` - name of a contract as Cairo shortstring. It is a name of the contract (part after `mod` keyword) e.g. `'HelloStarknet'`

```rust
use result::ResultTrait;

#[test]
fn test_declare() {
    let class_hash = declare('HelloStarknet');
    // ...
}
```
