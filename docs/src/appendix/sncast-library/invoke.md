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
- `fee_settings` - fee settings for the transaction.
- `nonce` - nonce for declare transaction. If not provided, nonce will be set automatically.

```rust
{{#include ../../../listings/invoke/src/lib.cairo}}
```

Structures used by the command:

```rust
#[derive(Drop, Clone, Debug)]
pub struct InvokeResult {
    pub transaction_hash: felt252,
}

#[derive(Drop, Copy, Debug, Serde, PartialEq)]
pub struct FeeSettings {
    pub max_fee: Option<felt252>,
    pub l1_gas: Option<u64>,
    pub l1_gas_price: Option<u128>,
    pub l2_gas: Option<u64>,
    pub l2_gas_price: Option<u128>,
    pub l1_data_gas: Option<u64>,
    pub l1_data_gas_price: Option<u128>,
}
```
