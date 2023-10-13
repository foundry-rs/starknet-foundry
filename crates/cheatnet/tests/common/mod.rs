use cairo_felt::Felt252;
use camino::Utf8PathBuf;
use cheatnet::cheatcodes::deploy::deploy;
use cheatnet::rpc::{call_contract, CallContractFailure, CallContractResult};
use cheatnet::state::{BlockifierState, CheatnetState};
use cheatnet::{cheatcodes::ContractArtifacts, rpc::CallContractOutput};
use conversions::StarknetConversions;
use starknet::core::utils::get_selector_from_name;
use starknet_api::core::ContractAddress;
use std::{collections::HashMap, str::FromStr};

use crate::common::scarb::{get_contracts_map, try_get_starknet_artifacts_path};

static TARGET_NAME: &str = "cheatnet_testing_contracts";

pub mod assertions;
pub mod cache;
pub mod scarb;
pub mod state;

pub fn recover_data(output: CallContractOutput) -> Vec<Felt252> {
    match output.result {
        CallContractResult::Success { ret_data, .. } => ret_data,
        CallContractResult::Failure(failure_type) => match failure_type {
            CallContractFailure::Panic { panic_data, .. } => panic_data,
            CallContractFailure::Error { msg, .. } => panic!("Call failed with message: {msg}"),
        },
    }
}

pub fn get_contracts() -> HashMap<String, ContractArtifacts> {
    let contracts_path = Utf8PathBuf::from_str("tests/contracts")
        .unwrap()
        .canonicalize_utf8()
        .unwrap();
    let artifacts_path = try_get_starknet_artifacts_path(&contracts_path, TARGET_NAME)
        .unwrap()
        .unwrap();
    get_contracts_map(&artifacts_path)
}

pub fn deploy_contract(
    blockifier_state: &mut BlockifierState,
    cheatnet_state: &mut CheatnetState,
    contract_name: &str,
    calldata: &[Felt252],
) -> ContractAddress {
    let contract = contract_name.to_owned().to_felt252();
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
) -> CallContractOutput {
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

#[must_use]
pub fn felt_selector_from_name(name: &str) -> Felt252 {
    let selector = get_selector_from_name(name).unwrap();
    Felt252::from_bytes_be(&selector.to_bytes_be())
}
