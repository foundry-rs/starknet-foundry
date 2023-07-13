# declare

> fn declare(contract: felt252) -> Result::<felt252, felt252>

Declares a contract and returns its class hash.

- `contract` - name of a contract as Cairo shortstring.

```rust
use result::ResultTrait;

#[test]
fn test_declare() {
    let class_hash = declare('mycontract').unwrap();
    // ...
}
```
