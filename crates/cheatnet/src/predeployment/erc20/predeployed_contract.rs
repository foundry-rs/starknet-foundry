use crate::{
    predeployment::predeployed_contract::PredeployedContract,
    runtime_extensions::forge_runtime_extension::cheatcodes::{
        generate_random_felt::generate_random_felt,
        storage::{map_entry_address, storage_key, variable_address},
    },
};
use conversions::felt::FromShortString;
use starknet_api::{
    core::{ClassHash, ContractAddress},
    state::StorageKey,
};
use starknet_types_core::felt::Felt;

use super::constructor_data::ERC20ConstructorData;

impl PredeployedContract {
    #[must_use]
    pub fn erc20(
        contract_address: ContractAddress,
        class_hash: ClassHash,
        raw_casm: &str,
        constructor_data: ERC20ConstructorData,
    ) -> Self {
        let ERC20ConstructorData {
            name,
            symbol,
            decimals,
            total_supply: (total_supply_low, total_supply_high),
            permitted_minter,
            upgrade_delay,
        } = constructor_data;

        let recipient = generate_random_felt();
        let recipient_balance_low_address = map_entry_address("ERC20_balances", &[recipient]);
        let recipient_balance_high_address =
            StorageKey(recipient_balance_low_address.try_into().unwrap())
                .next_storage_key()
                .unwrap();

        let storage_kv_updates = [
            // name
            (
                storage_key(variable_address("ERC20_name")).unwrap(),
                Felt::from_short_string(&name).unwrap(),
            ),
            // symbol
            (
                storage_key(variable_address("ERC20_symbol")).unwrap(),
                Felt::from_short_string(&symbol).unwrap(),
            ),
            // decimals
            (
                storage_key(variable_address("ERC20_decimals")).unwrap(),
                Felt::from(decimals),
            ),
            // total_supply low
            (
                storage_key(variable_address("ERC20_total_supply")).unwrap(),
                Felt::from(total_supply_low),
            ),
            // total_supply high
            (
                storage_key(variable_address("ERC20_total_supply"))
                    .unwrap()
                    .next_storage_key()
                    .unwrap(),
                Felt::from(total_supply_high),
            ),
            // recipient balance low
            (
                storage_key(recipient_balance_low_address).unwrap(),
                Felt::from(total_supply_low),
            ),
            // recipient balance high
            (
                storage_key(**recipient_balance_high_address).unwrap(),
                Felt::from(total_supply_high),
            ),
            // permitted_minter
            (
                storage_key(variable_address("permitted_minter")).unwrap(),
                **permitted_minter,
            ),
            // upgrade_delay
            (
                storage_key(variable_address("upgrade_delay")).unwrap(),
                Felt::from(upgrade_delay),
            ),
        ]
        .to_vec();

        Self::new(contract_address, class_hash, raw_casm, storage_kv_updates)
    }
}
