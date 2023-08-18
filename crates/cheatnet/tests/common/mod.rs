use cairo_felt::Felt252;
use camino::Utf8PathBuf;
use cheatnet::{conversions::felt_from_short_string, CheatnetState};
use starknet::core::utils::get_selector_from_name;
use starknet_api::core::ContractAddress;
use std::str::FromStr;

use crate::common::scarb::{get_contracts_map, try_get_starknet_artifacts_path};

static TARGET_NAME: &str = "cheatnet_testing_contracts";

pub mod assert;
pub mod scarb;
pub mod state;

pub fn deploy_contract(
    state: &mut CheatnetState,
    contract_name: &str,
    calldata: &[Felt252],
) -> ContractAddress {
    let contract = felt_from_short_string(contract_name);

    let contracts_path = Utf8PathBuf::from_str("tests/contracts")
        .unwrap()
        .canonicalize_utf8()
        .unwrap();
    let artifacts_path = try_get_starknet_artifacts_path(&contracts_path, TARGET_NAME)
        .unwrap()
        .unwrap();
    let contracts = get_contracts_map(&artifacts_path);

    let class_hash = state.declare(&contract, &contracts).unwrap();
    state.deploy(&class_hash, calldata).unwrap()
}

pub fn get_felt_selector_from_name(name: &str) -> Felt252 {
    let selector = get_selector_from_name(name).unwrap();
    Felt252::from_bytes_be(&selector.to_bytes_be())
}
