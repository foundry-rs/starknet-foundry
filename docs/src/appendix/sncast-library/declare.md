# `declare`

> `fn declare(contract_name: ByteArray, max_fee: Option<felt252>, nonce: Option<felt252>) -> DeclareResult`

Declares a contract and returns `DeclareResult`.

```rust
#[derive(Drop, Clone, Debug)]
pub struct DeclareResult {
    pub class_hash: ClassHash,
    pub transaction_hash: felt252,
}
```

- `contract_name` - name of a contract as Cairo string. It is a name of the contract (part after `mod` keyword) e.g. `"HelloStarknet"`.
- `max_fee` - max fee for declare transaction. If not provided, max fee will be automatically estimated.
- `nonce` - nonce for declare transaction. If not provided, nonce will be set automatically.

```rust
use sncast_std::{declare, DeclareResult};

fn main() {
    let max_fee = 9999999;
    let declare_result = declare("HelloStarknet", Option::Some(max_fee), Option::None);

    let class_hash = declare_result.class_hash;
    class_hash_to_felt252(class_hash).print();
    
    println!("declare_result: {}", declare_result);
    println!("debug declare_result: {:?}", declare_result);
}
```
