use crate::common::{get_contracts, state::create_cached_state};
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::{
    declare, get_class_hash,
};
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::CheatcodeError;
use runtime::EnhancedHintError;
use scarb_api::StarknetContractArtifacts;
use starknet_api::core::ClassHash;
use std::collections::HashMap;

fn get_contract_class_hash(
    contract_name: &str,
    contracts: &HashMap<String, StarknetContractArtifacts>,
) -> ClassHash {
    let contract = contracts.get(contract_name).unwrap();
    let sierra_class = serde_json::from_str(&contract.sierra).unwrap();
    get_class_hash(&sierra_class).unwrap()
}

#[test]
fn declare_simple() {
    let contract_name = "HelloStarknet";

    let mut cached_state = create_cached_state();

    let contracts_data = get_contracts();

    let class_hash = declare(&mut cached_state, contract_name, &contracts_data).unwrap();
    let expected_class_hash = get_contract_class_hash(contract_name, &contracts_data.contracts);

    assert_eq!(class_hash, expected_class_hash);
}

#[test]
fn declare_multiple() {
    let contract_names = vec!["HelloStarknet", "ConstructorSimple"];

    let mut cached_state = create_cached_state();

    let contracts_data = get_contracts();

    for contract_name in contract_names {
        let class_hash = declare(&mut cached_state, contract_name, &contracts_data).unwrap();
        let expected_class_hash = get_contract_class_hash(contract_name, &contracts_data.contracts);
        assert_eq!(class_hash, expected_class_hash);
    }
}

#[test]
fn declare_same_contract() {
    let contract_name = "HelloStarknet";

    let mut cached_state = create_cached_state();

    let contracts_data = get_contracts();

    let class_hash = declare(&mut cached_state, contract_name, &contracts_data).unwrap();
    let expected_class_hash = get_contract_class_hash(contract_name, &contracts_data.contracts);
    assert_eq!(class_hash, expected_class_hash);

    let output = declare(&mut cached_state, contract_name, &contracts_data);

    assert!(matches!(output, Err(CheatcodeError::Recoverable(_))));
}

#[test]
fn declare_non_existent() {
    let contract_name = "GoodbyeStarknet";

    let mut cached_state = create_cached_state();

    let contracts_data = get_contracts();

    let output = declare(&mut cached_state, contract_name, &contracts_data);

    assert!(match output {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Anyhow(msg))) => {
            let msg = msg.to_string();
            msg.contains("Failed") && msg.contains(contract_name)
        }
        _ => false,
    });
}
