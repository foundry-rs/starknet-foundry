use std::collections::HashMap;

use crate::common::state::create_cheatnet_state;
use crate::common::{get_contracts, state::create_cached_state};
use cheatnet::cheatcodes::declare::get_class_hash;
use cheatnet::cheatcodes::{CheatcodeError, EnhancedHintError};
use conversions::StarknetConversions;
use scarb_artifacts::StarknetContractArtifacts;
use starknet_api::core::ClassHash;
use test_case::test_case;

fn get_contract_class_hash(
    contract_name: &str,
    contracts: &HashMap<String, StarknetContractArtifacts>,
) -> ClassHash {
    let sierra = contracts.get(contract_name).unwrap();
    get_class_hash(sierra.sierra.as_str()).unwrap()
}

#[test_case("HelloStarknet")]
fn declare_simple(contract_name: &str) {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, _) = create_cheatnet_state(&mut cached_state);

    let contract = contract_name.to_owned().to_felt252();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();
    let expected_class_hash = get_contract_class_hash(contract_name, &contracts);

    assert_eq!(class_hash, expected_class_hash);
}

#[test_case(vec!["HelloStarknet", "ConstructorSimple"])]
fn declare_multiple(contract_names: Vec<&str>) {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, _) = create_cheatnet_state(&mut cached_state);

    let contracts = get_contracts();

    for contract_name in contract_names {
        let contract = contract_name.to_owned().to_felt252();
        let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();
        let expected_class_hash = get_contract_class_hash(contract_name, &contracts);
        assert_eq!(class_hash, expected_class_hash);
    }
}

#[test_case("HelloStarknet", "is already declared")]
fn declare_same_contract(contract_name: &str, error_msg: &str) {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, _) = create_cheatnet_state(&mut cached_state);

    let contract = contract_name.to_owned().to_felt252();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();
    let expected_class_hash = get_contract_class_hash(contract_name, &contracts);
    assert_eq!(class_hash, expected_class_hash);

    let output = blockifier_state.declare(&contract, &contracts);

    assert!(match output {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Anyhow(msg))) => {
            msg.to_string().contains(error_msg)
        }
        _ => false,
    });
}

#[test_case("GoodbyeStarknet")]
fn declare_non_existant(contract_name: &str) {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, _) = create_cheatnet_state(&mut cached_state);

    let contract = contract_name.to_owned().to_felt252();
    let contracts = get_contracts();

    let output = blockifier_state.declare(&contract, &contracts);

    assert!(match output {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Anyhow(msg))) => {
            msg.to_string().contains("Failed") && msg.to_string().contains(contract_name)
        }
        _ => false,
    });
}
