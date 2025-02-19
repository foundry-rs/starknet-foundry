use std::collections::HashMap;
use std::sync::Arc;

use blockifier::execution::contract_class::ContractClassV1;

use blockifier::execution::contract_class::ContractClass;
use blockifier::state::cached_state::CachedState;
use conversions::felt::FromShortString;

use crate::runtime_extensions::forge_runtime_extension::cheatcodes::storage::calculate_variable_address;
use crate::runtime_extensions::forge_runtime_extension::cheatcodes::storage::store;
use crate::state::ExtendedStateReader;
use starknet_types_core::felt::Felt;

use crate::data::contract_class_no_entrypoints::NO_ENTRYPOINTS_CASM;
use crate::data::strk_erc20_lockable::STRK_ERC_20_LOCKABLE_CASM;
use blockifier::execution::entry_point::{CallEntryPoint, CallType};
use conversions::string::TryFromHexStr;
use conversions::IntoConv;
use runtime::starknet::context::ERC20_CONTRACT_ADDRESS;
use runtime::starknet::state::DictStateReader;
use starknet::core::utils::get_selector_from_name;
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::{core::ContractAddress, transaction::Calldata};

pub const MAX_FEE: u128 = 1_000_000 * 100_000_000_000; // 1000000 * min_gas_price.
pub const INITIAL_BALANCE: u128 = 10 * MAX_FEE;

// Mocked class hashes, those are not checked anywhere
pub const TEST_CLASS_HASH: &str = "0x110";
pub const TEST_ACCOUNT_CONTRACT_CLASS_HASH: &str = "0x111";
pub const TEST_EMPTY_CONTRACT_CLASS_HASH: &str = "0x112";
pub const TEST_FAULTY_ACCOUNT_CONTRACT_CLASS_HASH: &str = "0x113";
pub const SECURITY_TEST_CLASS_HASH: &str = "0x114";
pub const TEST_ERC20_CONTRACT_CLASS_HASH: &str = "0x1010";

pub const TEST_CONTRACT_CLASS_HASH: &str = "0x117";
pub const TEST_ENTRY_POINT_SELECTOR: &str = "TEST_CONTRACT_SELECTOR";
// snforge_std/src/cheatcodes.cairo::test_address
pub const TEST_ADDRESS: &str = "0x01724987234973219347210837402";

pub const STRK_CONTRACT_ADDRESS: &str =
    "0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d";
pub const STRK_CLASS_HASH: &str =
    "0x04ad3c1dc8413453db314497945b6903e1c766495a1e60492d44da9c2a986e4b";

#[must_use]
pub fn get_contract_class(class: &str) -> ContractClass {
    ContractClass::V1(
        ContractClassV1::try_from_json_string(class)
            .expect("Could not create dummy contract from raw"),
    )
}

// Creates a state with predeployed account and erc20 used to send transactions during tests.
// Deployed contracts are cairo 0 contracts
// Account does not include validations
#[must_use]
pub fn build_testing_state() -> DictStateReader {
    let test_erc20_class_hash =
        TryFromHexStr::try_from_hex_str(TEST_ERC20_CONTRACT_CLASS_HASH).unwrap();
    let test_contract_class_hash =
        TryFromHexStr::try_from_hex_str(TEST_CONTRACT_CLASS_HASH).unwrap();

    let strk_contract_address = TryFromHexStr::try_from_hex_str(STRK_CONTRACT_ADDRESS).unwrap();
    let strk_class_hash = TryFromHexStr::try_from_hex_str(STRK_CLASS_HASH).unwrap();

    let class_hash_to_class = HashMap::from([
        // This dummy test contract class hash and class is put here only to satisfy blockifier
        // this class is not used and the test contract cannot be called
        (
            test_contract_class_hash,
            get_contract_class(NO_ENTRYPOINTS_CASM),
        ),
        (
            strk_class_hash,
            get_contract_class(STRK_ERC_20_LOCKABLE_CASM),
        ),
    ]);

    let test_erc20_address = TryFromHexStr::try_from_hex_str(ERC20_CONTRACT_ADDRESS).unwrap();
    let test_address = TryFromHexStr::try_from_hex_str(TEST_ADDRESS).unwrap();
    let address_to_class_hash = HashMap::from([
        (test_erc20_address, test_erc20_class_hash),
        (test_address, test_contract_class_hash),
        (strk_contract_address, strk_class_hash),
    ]);

    DictStateReader {
        address_to_class_hash,
        class_hash_to_class,
    }
}

#[must_use]
pub fn build_test_entry_point() -> CallEntryPoint {
    let test_selector = get_selector_from_name(TEST_ENTRY_POINT_SELECTOR).unwrap();
    let entry_point_selector = test_selector.into_();
    CallEntryPoint {
        class_hash: None,
        code_address: Some(TryFromHexStr::try_from_hex_str(TEST_ADDRESS).unwrap()),
        entry_point_type: EntryPointType::External,
        entry_point_selector,
        calldata: Calldata(Arc::new(vec![])),
        storage_address: TryFromHexStr::try_from_hex_str(TEST_ADDRESS).unwrap(),
        caller_address: ContractAddress::default(),
        call_type: CallType::Call,
        initial_gas: u64::MAX,
    }
}

pub fn setup_predeployed_strk_token(state: &mut CachedState<ExtendedStateReader>) {
    let storage_values = HashMap::from([
        (
            calculate_variable_address(Felt::from_short_string("ERC20_name").unwrap(), None),
            Felt::from_short_string("STRK").unwrap(),
        ),
        (
            calculate_variable_address(Felt::from_short_string("ERC20_symbol").unwrap(), None),
            Felt::from_short_string("STRK").unwrap(),
        ),
        (
            calculate_variable_address(Felt::from_short_string("ERC20_decimals").unwrap(), None),
            18.into(),
        ),
        (
            calculate_variable_address(
                Felt::from_short_string("ERC20_total_supply").unwrap(),
                None,
            ),
            Felt::from_dec_str("1294321502661951977636254715").unwrap(),
        ),
    ]);

    let strk_address: ContractAddress =
        TryFromHexStr::try_from_hex_str(STRK_CONTRACT_ADDRESS).unwrap();
    for (key, value) in storage_values {
        store(state, strk_address, key, value).expect("Could not store");
    }
}
