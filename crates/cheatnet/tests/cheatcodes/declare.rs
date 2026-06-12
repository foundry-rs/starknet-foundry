use crate::common::assertions::ClassHashAssert;
use crate::common::{get_contracts, state::create_cached_state};
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::CheatcodeError;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::{
    DeclareResult, declare,
};
use runtime::EnhancedHintError;

#[test]
fn declare_simple() {
    let contract_name = "HelloStarknet";

    let mut cached_state = create_cached_state();

    let contracts_data = get_contracts();

    let class_hash = declare(&mut cached_state, contract_name, &contracts_data)
        .unwrap()
        .unwrap_success();
    let expected_class_hash = &contracts_data
        .resolve_by_name(contract_name)
        .unwrap()
        .class_hash;

    assert_eq!(class_hash, *expected_class_hash);
}

#[test]
fn declare_multiple() {
    let contract_names = vec!["HelloStarknet", "ConstructorSimple"];

    let mut cached_state = create_cached_state();

    let contracts_data = get_contracts();

    for contract_name in contract_names {
        let class_hash = declare(&mut cached_state, contract_name, &contracts_data)
            .unwrap()
            .unwrap_success();
        let expected_class_hash = &contracts_data
            .resolve_by_name(contract_name)
            .unwrap()
            .class_hash;
        assert_eq!(class_hash, *expected_class_hash);
    }
}

#[test]
fn declare_same_contract() {
    let contract_name = "HelloStarknet";

    let mut cached_state = create_cached_state();

    let contracts_data = get_contracts();

    let class_hash = declare(&mut cached_state, contract_name, &contracts_data)
        .unwrap()
        .unwrap_success();
    let expected_class_hash = &contracts_data
        .resolve_by_name(contract_name)
        .unwrap()
        .class_hash;
    assert_eq!(class_hash, *expected_class_hash);

    let output = declare(&mut cached_state, contract_name, &contracts_data);

    assert!(
        matches!(output, Ok(DeclareResult::AlreadyDeclared(class_hash)) if  class_hash == *expected_class_hash)
    );
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

#[test]
fn declare_ambiguous_name() {
    let contract_name = "HelloStarknet";
    let duplicate_module_path = "duplicate::HelloStarknet".to_string();

    let mut cached_state = create_cached_state();

    // Introduce a second contract sharing the name `HelloStarknet` under a distinct module path,
    // making the name ambiguous.
    let mut contracts_data = get_contracts();
    let existing = contracts_data
        .contracts
        .values()
        .find(|contract| contract.name == contract_name)
        .expect("HelloStarknet should be present in the test fixtures")
        .clone();
    contracts_data
        .contracts
        .insert(duplicate_module_path.clone(), existing);

    let output = declare(&mut cached_state, contract_name, &contracts_data);

    assert!(match output {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Anyhow(msg))) => {
            let msg = msg.to_string();
            msg.contains("Multiple contracts found")
                && msg.contains(contract_name)
                && msg.contains(&duplicate_module_path)
        }
        _ => false,
    });
}
