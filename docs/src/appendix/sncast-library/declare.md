# `declare`

> `pub fn declare(contract_name: ByteArray, fee_settings: FeeSettings, nonce: Option<felt252>) -> Result<DeclareResult, ScriptCommandError>`

Declares a contract and returns `DeclareResult`.

```rust
#[derive(Drop, Clone, Debug)]
pub struct DeclareResult {
    pub class_hash: ClassHash,
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

- `contract_name` - name of a contract as Cairo string. It is a name of the contract (part after `mod` keyword) e.g. `"HelloStarknet"`.
- `fee_settings` - fee settings for the transaction. Can be `Eth` or `Strk`. Read more about it [here](../../starknet/fees-and-versions.md)
- `nonce` - nonce for declare transaction. If not provided, nonce will be set automatically.

```rust
use sncast_std::{declare, DeclareResult, FeeSettings, EthFeeSettings};

fn main() {
    let max_fee = 9999999;
    let declare_result = declare(
        "HelloStarknet",
        FeeSettings::Eth(EthFeeSettings { max_fee: Option::Some(max_fee) }),
        Option::None
    )
        .expect('declare failed');

    println!("declare_result: {}", declare_result);
    println!("debug declare_result: {:?}", declare_result);
}

```
