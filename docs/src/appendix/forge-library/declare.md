# `declare`

> `fn declare(contract: felt252) -> ContractClass`

Declares a contract and returns `ContractClass`.
Functions [deploy](./deploy.md) and [precalculate_address](./precalculate_address.md) can be called on this struct.

```rust
#[derive(Drop, Clone)]
struct ContractClass {
    class_hash: ClassHash,
}
```

- `contract` - name of a contract as Cairo shortstring. It is a name of the contract (part after `mod` keyword) e.g. `'HelloStarknet'`

```rust
use result::ResultTrait;
use snforge_std::declare;

#[test]
fn test_declare() {
    let contract = declare('HelloStarknet');
    // ...
}
```
