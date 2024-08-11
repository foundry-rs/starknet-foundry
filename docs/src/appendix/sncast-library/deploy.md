# `deploy`

> `pub fn deploy(
    class_hash: ClassHash,
    constructor_calldata: Option<ByteArray>,
    salt: Option<felt252>,
    unique: bool,
    fee_settings: FeeSettings,
    nonce: Option<felt252>
) -> Result<DeployResult, ScriptCommandError>`

Deploys a contract and returns `DeployResult`.

```rust
#[derive(Drop, Clone, Debug)]
pub struct DeployResult {
    pub contract_address: ContractAddress,
    pub transaction_hash: felt252,
}

#[derive(Drop, Clone, Debug, Serde, PartialEq)]
pub enum FeeSettings {
    Eth: EthFeeSettings,
    Strk: StrkFeeSettings
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

- `class_hash` - class hash of a contract to deploy.
- `constructor_calldata` - calldata for the contract constructor in form of Cairo-like expression. Should be in format `"{ arguments }"`.
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

- `salt` - salt for the contract address.
- `unique` - determines if salt should be further modified with the account address.
- `fee_settings` - fee settings for the transaction. Can be `Eth` or `Strk`. Read more about it [here](../../starknet/fees-and-versions.md) 
- `nonce` - nonce for declare transaction. If not provided, nonce will be set automatically.

```rust
use sncast_std::{deploy, DeployResult, FeeSettings, EthFeeSettings};

fn main() {
    let max_fee = 9999999;
    let salt = 0x1;
    let nonce = 0x1;
    let class_hash: ClassHash = 0x03a8b191831033ba48ee176d5dde7088e71c853002b02a1cfa5a760aa98be046
        .try_into()
        .expect('Invalid class hash value');

    let deploy_result = deploy(
        class_hash,
        Option::None,
        Option::Some(salt),
        true,
        FeeSettings::Eth(EthFeeSettings {max_fee: Option::Some(max_fee)}),
        Option::Some(deploy_nonce)
    ).expect('deploy failed');

    println!("deploy_result: {}", deploy_result);
    println!("debug deploy_result: {:?}", deploy_result);
}
```
