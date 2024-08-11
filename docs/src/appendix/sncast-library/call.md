# `call`

> `pub fn call(
    contract_address: ContractAddress, function_selector: felt252, calldata: Option<ByteArray>
) -> Result<CallResult, ScriptCommandError>`

Calls a contract and returns `CallResult`.

```rust
#[derive(Drop, Clone, Debug)]
pub struct CallResult {
    pub data: Array::<felt252>,
}
```

- `contract_address` - address of the contract to call.
- `function_selector` - the selector of the function to call.
- `calldata` - inputs to the function to be called in form of Cairo-like expression. Should be in format `"{ arguments }"`.
  Supported argument types:

| Argument type                       | Valid expressions                                                  |
|-------------------------------------|--------------------------------------------------------------------|
| numerical value (felt, u8, i8 etc.) | `0x1`, `2_u8`, `-3`                                                |
| shortstring                         | `'value'`                                                          |
| string (ByteArray)                  | `"value"`                                                          |
| boolean value                       | `true`, `false`                                                    |
| struct                              | `Struct { field_one: 0x1 }`, `path::to::Struct { field_one: 0x1 }` |
| enum                                | `Enum::One`, `Enum::Two(123)`, `path::to::Enum::Three`             |
| array                               | `array![0x1, 0x2, 0x3]`                                            |
| tuple                               | `(0x1, array![2], Struct { field: 'three' })`                      |


```rust
use sncast_std::{call, CallResult};
use starknet::{ContractAddress};

fn main() {
    let contract_address: ContractAddress = 0x1e52f6ebc3e594d2a6dc2a0d7d193cb50144cfdfb7fdd9519135c29b67e427
        .try_into()
        .expect('Invalid contract address value');

    let call_result = call(contract_address, selector!("get"), "{ 0x1 }").expect('call failed');
    println!("call_result: {}", call_result);
    println!("debug call_result: {:?}", call_result);
}
```
