# `declare`

> `fn declare(contract: felt252) -> ClassHash`

Declares a contract and returns its class hash.

- `contract` - name of a contract as Cairo shortstring. It is a name of the contract (part after `mod` keyword) e.g. `'HelloStarknet'`

```rust
use result::ResultTrait;
use cheatcodes::declare;

#[test]
fn test_declare() {
    let class_hash = declare('HelloStarknet');
    // ...
}
```
