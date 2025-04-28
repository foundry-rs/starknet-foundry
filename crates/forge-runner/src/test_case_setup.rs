use blockifier::state::cached_state::CachedState;
use cheatnet::{
    constants::{STRK_CLASS_HASH, STRK_CONTRACT_ADDRESS, contract_class_strk},
    runtime_extensions::forge_runtime_extension::cheatcodes::{
        generate_random_felt::generate_random_felt,
        storage::{map_entry_address, storage_key, variable_address},
    },
    state::ExtendedStateReader,
};
use conversions::{felt::FromShortString, string::TryFromHexStr};
use starknet_api::{core::ContractAddress, state::StorageKey};
use starknet_types_core::felt::Felt;

// fn declare_token(
//     cached_state: &mut CachedState<ExtendedStateReader>,
//     class_hash: ClassHash,
//     casm: &str,
// ) {
//     let contract_class = RunnableCompiledClass::V1(
//         CompiledClassV1::try_from_json_string(casm, SierraVersion::LATEST).unwrap(),
//     );
//     declare_with_contract_class(cached_state, contract_class, class_hash)
//         .expect("Failed to declare class");
// }

// fn deploy_token(
//     syscall_handler: &mut SyscallHintProcessor,
//     cheatnet_state: &mut CheatnetState,
//     class_hash: ClassHash,
//     contract_address: ContractAddress,
//     constructor_calldata: &[Felt],
// ) -> bool {
//     let deploy_result = deploy_at(
//         syscall_handler,
//         cheatnet_state,
//         &class_hash,
//         constructor_calldata,
//         contract_address,
//         true,
//     );

//     // It's possible that token can be already deployed (forking)
//     deploy_result.is_ok()
// }

// pub fn declare_token_strk(cached_state: &mut CachedState<ExtendedStateReader>) {
//     let class_hash = ClassHash::try_from_hex_str(STRK_CLASS_HASH).unwrap();
//     declare_token(cached_state, class_hash, STRK_ERC20_CASM);
// }

// pub fn deploy_token_strk(
//     syscall_handler: &mut SyscallHintProcessor,
//     cheatnet_state: &mut CheatnetState,
// ) -> bool {
//     let class_hash = ClassHash::try_from_hex_str(STRK_CLASS_HASH).unwrap();
//     let contract_address = ContractAddress::try_from_hex_str(STRK_CONTRACT_ADDRESS).unwrap();
//     let constructor_calldata = strk_constructor_calldata();
//     deploy_token(
//         syscall_handler,
//         cheatnet_state,
//         class_hash,
//         contract_address,
//         &constructor_calldata,
//     )
// }

// const STARKNET_DOMAIN_TYPE_HASH = Felt::from_hex_str(
//         "0x0c4f2a1b3d5e7f8e9b6a2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8g9h0i1j2k3l",
//     )
//     .unwrap();

//     const DAPP_NAME = Felt::from_hex_str(
//         "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
//     )
//     .unwrap();

//     const DAPP_VERSION = Felt::from_hex_str(
//         "0x
//         1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
//     )
//     .unwrap();
// pub const STARKNET_DOMAIN_TYPE_HASH: &str =
//     "0x0c4f2a1b3d5e7f8e9b6a2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8g9h0i1j2k3l";
// pub const DAPP_NAME: &str = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
// pub const DAPP_VERSION: &str = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";

pub fn is_strk_deployed(state_reader: &mut ExtendedStateReader) -> bool {
    let strk_contract_address = ContractAddress::try_from_hex_str(STRK_CONTRACT_ADDRESS).unwrap();
    if let Some(ref fork_state_reader) = state_reader.fork_state_reader {
        let class_hash = fork_state_reader
            .cache
            .borrow()
            .get_class_hash_at(&strk_contract_address);
        return class_hash.is_some();
    }

    false
}

pub fn add_strk_to_dict_state_reader(cached_state: &mut CachedState<ExtendedStateReader>) {
    let strk_contract_address = ContractAddress::try_from_hex_str(STRK_CONTRACT_ADDRESS).unwrap();
    let strk_class_hash = TryFromHexStr::try_from_hex_str(STRK_CLASS_HASH).unwrap();

    cached_state
        .state
        .dict_state_reader
        .address_to_class_hash
        .insert(strk_contract_address, strk_class_hash);

    cached_state
        .state
        .dict_state_reader
        .class_hash_to_class
        .insert(strk_class_hash, contract_class_strk());

    let recipient = generate_random_felt();
    let recipient_balance_low_address = map_entry_address("ERC20_balances", &[recipient]);
    let recipient_balance_high_address =
        StorageKey(recipient_balance_low_address.try_into().unwrap())
            .next_storage_key()
            .unwrap();
    let total_supply_low = 55_401_946_922_417_748_965_830_181_u128;

    // Update STRK storage to mimic constructor behavior
    let storage_entries_and_values_to_update = [
        // name
        (
            (
                strk_contract_address,
                storage_key(variable_address("ERC20_name")).unwrap(),
            ),
            Felt::from_short_string("STRK").unwrap(),
        ),
        // symbol
        (
            (
                strk_contract_address,
                storage_key(variable_address("ERC20_symbol")).unwrap(),
            ),
            Felt::from_short_string("STRK").unwrap(),
        ),
        // decimals
        (
            (
                strk_contract_address,
                storage_key(variable_address("ERC20_decimals")).unwrap(),
            ),
            Felt::from(18),
        ),
        // total_supply low
        (
            (
                strk_contract_address,
                storage_key(variable_address("ERC20_total_supply")).unwrap(),
            ),
            Felt::from(total_supply_low),
        ),
        // total_supply high
        (
            (
                strk_contract_address,
                storage_key(map_entry_address("ERC20_total_supply", &[Felt::ONE])).unwrap(),
            ),
            Felt::ZERO,
        ),
        // recipient balance low
        (
            (
                strk_contract_address,
                storage_key(recipient_balance_low_address).unwrap(),
            ),
            Felt::from(total_supply_low),
        ),
        // recipient balance high
        (
            (
                strk_contract_address,
                storage_key(**recipient_balance_high_address).unwrap(),
            ),
            Felt::ZERO,
        ),
        // permitted_minter
        (
            (
                strk_contract_address,
                storage_key(variable_address("permitted_minter")).unwrap(),
            ),
            generate_random_felt(),
        ),
        // skip initializing roles
        // upgrade_delay
        (
            (
                strk_contract_address,
                storage_key(variable_address("upgrade_delay")).unwrap(),
            ),
            Felt::ZERO,
        ),
        // TODO: Decide if we want to write `domain_hash` to storage
        // it enforces us to read chain_id if the test uses forking, hence
        // this is a potential performance issue
    ];

    for (entry, value) in &storage_entries_and_values_to_update {
        cached_state
            .state
            .dict_state_reader
            .storage_view
            .insert(*entry, *value);
    }
}
