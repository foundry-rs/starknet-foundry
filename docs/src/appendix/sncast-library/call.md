# `call`

> `fn call(
    contract_address: ContractAddress, function_selector: felt252, calldata: Array::<felt252>
) -> CallResult`

Calls a contract and returns `CallResult`.

```rust
#[derive(Drop, Clone, Debug)]
pub struct CallResult {
    pub data: Array::<felt252>,
}
```

- `contract_address` - address of the contract to call.
- `function_selector` - the selector of the function to call, as Cairo shortstring.
- `calldata` - inputs to the function to be called.

```rust
use sncast_std::{call, CallResult};
use starknet::{ContractAddress};

fn main() {
    let contract_address: ContractAddress = 0x1e52f6ebc3e594d2a6dc2a0d7d193cb50144cfdfb7fdd9519135c29b67e427
        .try_into()
        .expect('Invalid contract address value');

    let call_result = call(contract_address, selector!("get"), array![0x1]);
    println!("call_result: {}", call_result);
    println!("debug call_result: {:?}", call_result);
}
```
