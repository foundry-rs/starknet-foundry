use blockifier::execution::entry_point::{CallEntryPoint, CallType};
use cairo_felt::Felt252;
use camino::Utf8PathBuf;
use cheatnet::constants::TEST_ADDRESS;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    call_entry_point, AddressOrClassHash, CallOutput,
};
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    CallFailure, CallResult,
};
use cheatnet::runtime_extensions::common::{create_entry_point_selector, create_execute_calldata};
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::deploy::deploy;
use cheatnet::state::{BlockifierState, CheatnetState};
use conversions::felt252::FromShortString;
use scarb_api::{get_contracts_map, StarknetContractArtifacts};
use starknet::core::utils::get_selector_from_name;
use starknet_api::core::ContractAddress;
use starknet_api::core::PatriciaKey;
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::hash::StarkHash;
use starknet_api::patricia_key;
use std::collections::HashMap;

pub mod assertions;
pub mod cache;
pub mod state;

pub fn recover_data(output: CallOutput) -> Vec<Felt252> {
    match output.result {
        CallResult::Success { ret_data, .. } => ret_data,
        CallResult::Failure(failure_type) => match failure_type {
            CallFailure::Panic { panic_data, .. } => panic_data,
            CallFailure::Error { msg, .. } => panic!("Call failed with message: {msg}"),
        },
    }
}

pub fn get_contracts() -> HashMap<String, StarknetContractArtifacts> {
    let scarb_metadata = scarb_metadata::MetadataCommand::new()
        .inherit_stderr()
        .manifest_path(Utf8PathBuf::from("tests/contracts/Scarb.toml"))
        .exec()
        .unwrap();

    let package = scarb_metadata.packages.first().unwrap();
    get_contracts_map(&scarb_metadata, &package.id).unwrap()
}

pub fn deploy_contract(
    blockifier_state: &mut BlockifierState,
    cheatnet_state: &mut CheatnetState,
    contract_name: &str,
    calldata: &[Felt252],
) -> ContractAddress {
    let contract = Felt252::from_short_string(contract_name).unwrap();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();
    deploy(blockifier_state, cheatnet_state, &class_hash, calldata)
        .unwrap()
        .contract_address
}

pub fn call_contract_getter_by_name(
    blockifier_state: &mut BlockifierState,
    cheatnet_state: &mut CheatnetState,
    contract_address: &ContractAddress,
    fn_name: &str,
) -> CallOutput {
    let selector = felt_selector_from_name(fn_name);
    let result = call_contract(
        blockifier_state,
        cheatnet_state,
        contract_address,
        &selector,
        vec![].as_slice(),
    )
    .unwrap();

    result
}

// This does contract call without the transaction layer. This way `call_contract` can return data and modify state.
// `call` and `invoke` on the transactional layer use such method under the hood.
pub fn call_contract(
    blockifier_state: &mut BlockifierState,
    cheatnet_state: &mut CheatnetState,
    contract_address: &ContractAddress,
    entry_point_selector: &Felt252,
    calldata: &[Felt252],
) -> anyhow::Result<CallOutput> {
    let entry_point_selector = create_entry_point_selector(entry_point_selector);
    let calldata = create_execute_calldata(calldata);

    let entry_point = CallEntryPoint {
        class_hash: None,
        code_address: Some(*contract_address),
        entry_point_type: EntryPointType::External,
        entry_point_selector,
        calldata,
        storage_address: *contract_address,
        caller_address: ContractAddress(patricia_key!(TEST_ADDRESS)),
        call_type: CallType::Call,
        initial_gas: u64::MAX,
    };

    call_entry_point(
        blockifier_state,
        cheatnet_state,
        entry_point,
        &AddressOrClassHash::ContractAddress(*contract_address),
    )
}

#[must_use]
pub fn felt_selector_from_name(name: &str) -> Felt252 {
    let selector = get_selector_from_name(name).unwrap();
    Felt252::from_bytes_be(&selector.to_bytes_be())
}
