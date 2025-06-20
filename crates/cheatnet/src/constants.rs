use starknet_api::contract_class::{ContractClass, SierraVersion};
use std::collections::HashMap;
use std::sync::Arc;

use blockifier::execution::entry_point::{CallType, ExecutableCallEntryPoint};
use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use cairo_lang_starknet_classes::compiler_version::current_sierra_version_id;
use conversions::IntoConv;
use conversions::string::TryFromHexStr;
use indoc::indoc;
use runtime::starknet::constants::{
    TEST_ADDRESS, TEST_CONTRACT_CLASS_HASH, TEST_ENTRY_POINT_SELECTOR,
};
use runtime::starknet::context::ERC20_CONTRACT_ADDRESS;
use runtime::starknet::state::DictStateReader;
use starknet::core::utils::get_selector_from_name;
use starknet_api::contract_class::EntryPointType;
use starknet_api::core::ClassHash;
use starknet_api::{core::ContractAddress, transaction::fields::Calldata};

// Mocked class hashes, those are not checked anywhere
pub const TEST_ERC20_CONTRACT_CLASS_HASH: &str = "0x1010";

fn contract_class_no_entrypoints() -> ContractClass {
    let raw_contract_class = indoc!(
        r#"{
          "prime": "0x800000000000011000000000000000000000000000000000000000000000001",
          "compiler_version": "2.4.0",
          "bytecode": [],
          "hints": [],
          "entry_points_by_type": {
            "EXTERNAL": [],
            "L1_HANDLER": [],
            "CONSTRUCTOR": []
          }
        }"#,
    );
    let casm_contract_class: CasmContractClass = serde_json::from_str(raw_contract_class)
        .expect("`raw_contract_class` should be valid casm contract class");

    ContractClass::V1((casm_contract_class, SierraVersion::default()))
}

#[must_use]
pub fn contract_class(raw_casm: &str, sierra_version: SierraVersion) -> ContractClass {
    let casm_contract_class: CasmContractClass =
        serde_json::from_str(raw_casm).expect("`raw_casm` should be valid casm contract class");

    ContractClass::V1((casm_contract_class, sierra_version))
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

    let class_hash_to_class = HashMap::from([
        // This is dummy put here only to satisfy blockifier
        // this class is not used and the test contract cannot be called
        (test_contract_class_hash, contract_class_no_entrypoints()),
    ]);

    let test_erc20_address = TryFromHexStr::try_from_hex_str(ERC20_CONTRACT_ADDRESS).unwrap();
    let test_address = TryFromHexStr::try_from_hex_str(TEST_ADDRESS).unwrap();
    let address_to_class_hash = HashMap::from([
        (test_erc20_address, test_erc20_class_hash),
        (test_address, test_contract_class_hash),
    ]);

    DictStateReader {
        address_to_class_hash,
        class_hash_to_class,
        ..Default::default()
    }
}

#[must_use]
pub fn build_test_entry_point() -> ExecutableCallEntryPoint {
    let test_selector = get_selector_from_name(TEST_ENTRY_POINT_SELECTOR).unwrap();
    let entry_point_selector = test_selector.into_();
    ExecutableCallEntryPoint {
        class_hash: ClassHash(TryFromHexStr::try_from_hex_str(TEST_CONTRACT_CLASS_HASH).unwrap()),
        code_address: Some(TryFromHexStr::try_from_hex_str(TEST_ADDRESS).unwrap()),
        entry_point_type: EntryPointType::External,
        entry_point_selector,
        calldata: Calldata(Arc::new(vec![])),
        storage_address: TryFromHexStr::try_from_hex_str(TEST_ADDRESS).unwrap(),
        caller_address: ContractAddress::default(),
        call_type: CallType::Call,
        initial_gas: i64::MAX as u64,
    }
}

#[must_use]
pub fn get_current_sierra_version() -> SierraVersion {
    let version_id = current_sierra_version_id();
    SierraVersion::new(
        version_id.major as u64,
        version_id.minor as u64,
        version_id.patch as u64,
    )
}
