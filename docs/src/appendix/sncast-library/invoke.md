# `invoke`

> `pub fn invoke(
    contract_address: ContractAddress,
    entry_point_selector: felt252,
    calldata: Array::<felt252>,
    fee_settings: FeeSettings,
    nonce: Option<felt252>
) -> Result<InvokeResult, ScriptCommandError>`

Invokes a contract and returns `InvokeResult`.

- `contract_address` - address of the contract to invoke.
- `entry_point_selector` - the selector of the function to invoke.
- `calldata` - inputs to the function to be invoked.
- `fee_settings` - fee settings for the transaction. Can be `Eth` or `Strk`. Read more about it [here](../../starknet/fees-and-versions.md)
- `nonce` - nonce for declare transaction. If not provided, nonce will be set automatically.

```rust
{{#include ../../../listings/sncast_library/scripts/invoke/src/lib.cairo}}
```

Structures used by the command:

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
