# `invoke`

> `pub fn invoke(
    contract_address: ContractAddress,
    entry_point_selector: felt252,
    calldata: Option<ByteArray>,
    fee_settings: FeeSettings,
    nonce: Option<felt252>
) -> Result<InvokeResult, ScriptCommandError>`

Invokes a contract and returns `InvokeResult`.

```rust
#[derive(Drop, Clone, Debug)]
pub struct InvokeResult {
    pub transaction_hash: felt252,
}

#[derive(Drop, Clone, Debug, Serde, PartialEq)]
pub struct EthFeeSettings {
    pub max_fee: Option<felt252>,
}

#[derive(Drop, Clone, Debug, Serde, PartialEq)]
pub struct StrkFeeSettings {
    pub max_fee: Option<felt252>,
    pub max_gas: Option<u64>,
    pub max_gas_unit_price: Option<u128>,
}
```

- `contract_address` - address of the contract to invoke.
- `entry_point_selector` - the selector of the function to invoke.
- `calldata` - inputs to the function to be invoked in form of Cairo-like expression. Should be in format `"{ arguments }"`.
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

- `fee_settings` - fee settings for the transaction. Can be `Eth` or `Strk`. Read more about it [here](../../starknet/fees-and-versions.md)
- `nonce` - nonce for declare transaction. If not provided, nonce will be set automatically.

```rust
use sncast_std::{invoke, InvokeResult, FeeSettings, EthFeeSettings};
use starknet::{ContractAddress};

fn main() {
    let contract_address: ContractAddress =
        0x1e52f6ebc3e594d2a6dc2a0d7d193cb50144cfdfb7fdd9519135c29b67e427
            .try_into()
            .expect('Invalid contract address value');

    let invoke_result = invoke(
        contract_address,
        selector!("put"),
        "{ 0x1, 0x2 }",
        FeeSettings::Eth(EthFeeSettings { max_fee: Option::None }),
        Option::None
    )
        .expect('invoke failed');

    println!("invoke_result: {}", invoke_result);
    println!("debug invoke_result: {:?}", invoke_result);
}

```
