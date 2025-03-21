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

#[generate_trait]
pub impl FeeSettingsImpl of FeeSettingsTrait {
    fn resource_bounds(
        l1_gas: u64,
        l1_gas_price: u128,
        l2_gas: u64,
        l2_gas_price: u128,
        l1_data_gas: u64,
        l2_data_gas_price: u128
    ) -> FeeSettings {
        FeeSettings {
            max_fee: Option::None,
            l1_gas: Option::Some(l1_gas),
            l1_gas_price: Option::Some(l1_gas_price),
            l2_gas: Option::Some(l2_gas),
            l2_gas_price: Option::Some(l2_gas_price),
            l1_data_gas: Option::Some(l1_data_gas),
            l2_data_gas_price: Option::Some(l2_data_gas_price),
        }
    }

    fn max_fee(max_fee: felt252) -> FeeSettings {
        FeeSettings {
            max_fee: Option::Some(max_fee),
            l1_gas: Option::None,
            l1_gas_price: Option::None,
            l2_gas: Option::None,
            l2_gas_price: Option::None,
            l1_data_gas: Option::None,
            l2_data_gas_price: Option::None,
        }
    }

    fn estimate() -> FeeSettings {
        FeeSettings {
            max_fee: Option::None,
            l1_gas: Option::None,
            l1_gas_price: Option::None,
            l2_gas: Option::None,
            l2_gas_price: Option::None,
            l1_data_gas: Option::None,
            l2_data_gas_price: Option::None,
        }
    }
}
```
