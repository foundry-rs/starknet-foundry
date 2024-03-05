# `invoke`

> `pub fn invoke(
    contract_address: ContractAddress,
    entry_point_selector: felt252,
    calldata: Array::<felt252>,
    max_fee: Option<felt252>,
    nonce: Option<felt252>
) -> Result<InvokeResult, ScriptCommandError>`

Invokes a contract and returns `InvokeResult`.

```rust
#[derive(Drop, Clone, Debug)]
pub struct InvokeResult {
    pub transaction_hash: felt252,
}
```

- `contract_address` - address of the contract to invoke.
- `entry_point_selector` - the selector of the function to invoke.
- `calldata` - inputs to the function to be invoked.
- `max_fee` - max fee for declare transaction. If not provided, max fee will be automatically estimated.
- `nonce` - nonce for declare transaction. If not provided, nonce will be set automatically.

```rust
use sncast_std::{invoke, InvokeResult};
use starknet::{ContractAddress};

fn main() {
    let contract_address: ContractAddress = 0x1e52f6ebc3e594d2a6dc2a0d7d193cb50144cfdfb7fdd9519135c29b67e427
        .try_into()
        .expect('Invalid contract address value');

    let invoke_result = invoke(
        contract_address, selector!("put"), array![0x1, 0x2], Option::None, Option::None
    ).expect('invoke failed');

    println!("invoke_result: {}", invoke_result);
    println!("debug invoke_result: {:?}", invoke_result);
}
```
