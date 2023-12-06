# `call`

> `fn call(
    contract_address: ContractAddress, function_name: felt252, calldata: Array::<felt252>
) -> CallResult`

Calls a contract and returns `CallResult`.

```rust
#[derive(Drop, Clone)]
struct CallResult {
    data: Array::<felt252>,
}
```

- `contract_address` - address of the contract to call.
- `function_name` - the name of the function to call, as Cairo shortstring.
- `calldata` - inputs to the function to be called.

```rust
use sncast_std::{call, CallResult};
use starknet::{ContractAddress, Felt252TryIntoContractAddress};
use debug::PrintTrait;

fn main() {
    let contract_address: ContractAddress = 0x1e52f6ebc3e594d2a6dc2a0d7d193cb50144cfdfb7fdd9519135c29b67e427
        .try_into()
        .expect('Invalid contract address value');

    let call_result = call(contract_address, 'get', array![0x1]);
    let first_item = *call_result.data[0];
    first_item.print();
}
```
