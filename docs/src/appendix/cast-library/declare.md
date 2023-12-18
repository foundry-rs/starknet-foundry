# `declare`

> `fn declare(contract_name: felt252, max_fee: Option<felt252>, nonce: Option<felt252>) -> DeclareResult`

Declares a contract and returns `DeclareResult`.

```rust
#[derive(Drop, Clone)]
struct DeclareResult {
    class_hash: ClassHash,
    transaction_hash: felt252,
}
```

- `contract_name` - name of a contract as Cairo shortstring. It is a name of the contract (part after `mod` keyword) e.g. `'HelloStarknet'`.
- `max_fee` - max fee for declare transaction. If not provided, max fee will be automatically estimated.
- `nonce` - nonce for declare transaction. If not provided, nonce will be set automatically.

```rust
use sncast_std::{declare, DeclareResult};
use debug::PrintTrait;
use starknet::class_hash_to_felt252

fn main() {
    let max_fee = 9999999;
    let declare_result = declare('HelloStarknet', Option::Some(max_fee), Option::None);

    let class_hash = declare_result.class_hash;
    class_hash_to_felt252(class_hash).print();
}
```
