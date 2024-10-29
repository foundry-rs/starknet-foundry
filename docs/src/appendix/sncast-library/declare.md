# `declare`

> `pub fn declare(contract_name: ByteArray, fee_settings: FeeSettings, nonce: Option<felt252>) -> Result<DeclareResult, ScriptCommandError>`

Declares a contract and returns `DeclareResult`.

- `contract_name` - name of a contract as Cairo string. It is a name of the contract (part after `mod` keyword) e.g. `"HelloStarknet"`.
- `fee_settings` - fee settings for the transaction. Can be `Eth` or `Strk`. Read more about it [here](../../starknet/fees-and-versions.md)
- `nonce` - nonce for declare transaction. If not provided, nonce will be set automatically.

```rust
{{#include ../../../listings/sncast_library/scripts/declare/src/lib.cairo}}
```

Structures used by the command:

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
