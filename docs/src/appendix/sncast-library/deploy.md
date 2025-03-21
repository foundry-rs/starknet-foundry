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

- `class_hash` - class hash of a contract to deploy.
- `constructor_calldata` - calldata for the contract constructor.
- `salt` - salt for the contract address.
- `unique` - determines if salt should be further modified with the account address.
- `fee_settings` - fee settings for the transaction.
- `nonce` - nonce for declare transaction. If not provided, nonce will be set automatically.

```rust
{{#include ../../../listings/deploy/src/lib.cairo}}
```
