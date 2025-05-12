use crate::{
    constants::{STRK_CLASS_HASH, STRK_CONTRACT_ADDRESS, STRK_ERC20_CASM, contract_class},
    runtime_extensions::forge_runtime_extension::cheatcodes::{
        generate_random_felt::generate_random_felt,
        storage::{map_entry_address, storage_key, variable_address},
    },
    state::ExtendedStateReader,
};
use conversions::{felt::FromShortString, string::TryFromHexStr};
use starknet_api::{core::ContractAddress, state::StorageKey};
use starknet_types_core::felt::Felt;

// All values are taken from https://starkscan.co/contract/0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d#contract-storage
// result of `variable_address("permitted_minter")` in the search bar for the key
const STRK_PERMITTED_MINTER: &str =
    "0x594c1582459ea03f77deaf9eb7e3917d6994a03c13405ba42867f83d85f085d";

// result of `variable_address("upgrade_delay")` in the search bar for the key
const STRK_UPGRADE_DELAY: u64 = 0;

pub fn deploy_strk_token(state_reader: &mut ExtendedStateReader) {
    let strk_contract_address = ContractAddress::try_from_hex_str(STRK_CONTRACT_ADDRESS).unwrap();
    let strk_class_hash = TryFromHexStr::try_from_hex_str(STRK_CLASS_HASH).unwrap();

    state_reader
        .dict_state_reader
        .address_to_class_hash
        .insert(strk_contract_address, strk_class_hash);

    state_reader
        .dict_state_reader
        .class_hash_to_class
        .insert(strk_class_hash, contract_class(STRK_ERC20_CASM));

    let recipient = generate_random_felt();
    let recipient_balance_low_address = map_entry_address("ERC20_balances", &[recipient]);
    let recipient_balance_high_address =
        StorageKey(recipient_balance_low_address.try_into().unwrap())
            .next_storage_key()
            .unwrap();
    let total_supply_low: u128 = 60_000_000_000_000_000_000_000_000;

    // Update STRK storage to mimic constructor behavior
    let storage_entries_and_values_to_update = [
        // name
        (
            storage_key(variable_address("ERC20_name")).unwrap(),
            Felt::from_short_string("STRK").unwrap(),
        ),
        // symbol
        (
            storage_key(variable_address("ERC20_symbol")).unwrap(),
            Felt::from_short_string("STRK").unwrap(),
        ),
        // decimals
        (
            storage_key(variable_address("ERC20_decimals")).unwrap(),
            Felt::from(18),
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
            Felt::ZERO,
        ),
        // recipient balance low
        (
            storage_key(recipient_balance_low_address).unwrap(),
            Felt::from(total_supply_low),
        ),
        // recipient balance high
        (
            storage_key(**recipient_balance_high_address).unwrap(),
            Felt::ZERO,
        ),
        // permitted_minter
        (
            storage_key(variable_address("permitted_minter")).unwrap(),
            Felt::try_from_hex_str(STRK_PERMITTED_MINTER).unwrap(),
        ),
        // skip initializing roles
        // upgrade_delay
        (
            storage_key(variable_address("upgrade_delay")).unwrap(),
            STRK_UPGRADE_DELAY.into(),
        ),
        // TODO: Decide if we want to write `domain_hash` to storage
        // it enforces us to read chain_id if the test uses forking, hence
        // this is a potential performance issue
    ];

    for (key, value) in &storage_entries_and_values_to_update {
        let entry = (strk_contract_address, *key);
        state_reader
            .dict_state_reader
            .storage_view
            .insert(entry, *value);
    }
}
