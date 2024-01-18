use std::collections::HashMap;

use crate::common::state::create_cheatnet_state;
use crate::common::{get_contracts, state::create_cached_state};
use cairo_felt::Felt252;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::get_class_hash;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::CheatcodeError;
use conversions::felt252::FromShortString;
use runtime::EnhancedHintError;
use scarb_api::StarknetContractArtifacts;
use starknet_api::core::ClassHash;

fn get_contract_class_hash(
    contract_name: &str,
    contracts: &HashMap<String, StarknetContractArtifacts>,
) -> ClassHash {
    let sierra = contracts.get(contract_name).unwrap();
    get_class_hash(sierra.sierra.as_str()).unwrap()
}

#[test]
fn declare_simple() {
    let contract_name = "HelloStarknet";

    let mut cached_state = create_cached_state();
    let (mut blockifier_state, _) = create_cheatnet_state(&mut cached_state);

    let contract = Felt252::from_short_string(contract_name).unwrap();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();
    let expected_class_hash = get_contract_class_hash(contract_name, &contracts);

    assert_eq!(class_hash, expected_class_hash);
}

#[test]
fn declare_multiple() {
    let contract_names = vec!["HelloStarknet", "ConstructorSimple"];

    let mut cached_state = create_cached_state();
    let (mut blockifier_state, _) = create_cheatnet_state(&mut cached_state);

    let contracts = get_contracts();

    for contract_name in contract_names {
        let contract = Felt252::from_short_string(contract_name).unwrap();
        let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();
        let expected_class_hash = get_contract_class_hash(contract_name, &contracts);
        assert_eq!(class_hash, expected_class_hash);
    }
}

#[test]
fn declare_same_contract() {
    let contract_name = "HelloStarknet";

    let mut cached_state = create_cached_state();
    let (mut blockifier_state, _) = create_cheatnet_state(&mut cached_state);

    let contract = Felt252::from_short_string(contract_name).unwrap();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();
    let expected_class_hash = get_contract_class_hash(contract_name, &contracts);
    assert_eq!(class_hash, expected_class_hash);

    let output = blockifier_state.declare(&contract, &contracts);

    assert!(match output {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Anyhow(msg))) => {
            msg.to_string().contains("is already declared")
        }
        _ => false,
    });
}

#[test]
fn declare_non_existent() {
    let contract_name = "GoodbyeStarknet";

    let mut cached_state = create_cached_state();
    let (mut blockifier_state, _) = create_cheatnet_state(&mut cached_state);

    let contract = Felt252::from_short_string(contract_name).unwrap();
    let contracts = get_contracts();

    let output = blockifier_state.declare(&contract, &contracts);

    assert!(match output {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Anyhow(msg))) => {
            msg.to_string().contains("Failed") && msg.to_string().contains(contract_name)
        }
        _ => false,
    });
}
