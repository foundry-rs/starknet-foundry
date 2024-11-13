# `deploy`

> `pub fn deploy(
    class_hash: ClassHash,
    constructor_calldata: Array::<felt252>,
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
- `constructor_calldata` - calldata for the contract constructor.
- `salt` - salt for the contract address.
- `unique` - determines if salt should be further modified with the account address.
- `fee_settings` - fee settings for the transaction. Can be `Eth` or `Strk`. Read more about it [here](../../starknet/fees-and-versions.md)
- `nonce` - nonce for declare transaction. If not provided, nonce will be set automatically.

```rust
{{#include ../../../listings/sncast_library/scripts/deploy/src/lib.cairo}}
```
