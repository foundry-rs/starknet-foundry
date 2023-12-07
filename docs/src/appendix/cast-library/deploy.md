# `deploy`

> `fn deploy(
    class_hash: ClassHash,
    constructor_calldata: Array::<felt252>,
    salt: Option<felt252>,
    unique: bool,
    max_fee: Option<felt252>,
    nonce: Option<felt252>
) -> DeployResult`

Deploys a contract and returns `DeployResult`.

```rust
#[derive(Drop, Clone)]
struct DeployResult {
    contract_address: ContractAddress,
    transaction_hash: felt252,
}
```

- `class_hash` - class hash of a contract to deploy.
- `constructor_calldata` - calldata for the contract constructor.
- `salt` - salt for the contract address.
- `unique` - determines if salt should be further modified with the account address.
- `max_fee` - max fee for declare transaction. If not provided, max fee will be automatically estimated.
- `nonce` - nonce for declare transaction. If not provided, nonce will be set automatically.

```rust
use sncast_std::{deploy, DeployResult};
use starknet::{ClassHash, Felt252TryIntoClassHash};
use debug::PrintTrait;

fn main() {
    let max_fee = 9999999;
    let salt = 0x1;
    let nonce = 0x1;
    let class_hash: ClassHash = 0x03a8b191831033ba48ee176d5dde7088e71c853002b02a1cfa5a760aa98be046
        .try_into()
        .expect('Invalid class hash value');

    let deploy_result = deploy(
        class_hash,
        ArrayTrait::new(),
        Option::Some(salt),
        true,
        Option::Some(max_fee),
        Option::Some(nonce)
    );

    deploy_result.contract_address.print();
}
```
