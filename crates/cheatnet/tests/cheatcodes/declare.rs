use crate::common::assertions::ClassHashAssert;
use crate::common::{get_contracts, state::create_cached_state};
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::CheatcodeError;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::{
    DeclareResult, declare,
};
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::contract_name_from_module_path;
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
        .resolve_contract(contract_name)
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
            .resolve_contract(contract_name)
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
        .resolve_contract(contract_name)
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
fn declare_by_module_path() {
    let contract_name = "HelloStarknet";

    let mut cached_state = create_cached_state();

    let contracts_data = get_contracts();

    // Look up the full module path under which `HelloStarknet` is registered.
    let module_path = contracts_data
        .contracts
        .iter()
        .find(|(module_path, _)| contract_name_from_module_path(*module_path) == contract_name)
        .map(|(module_path, _)| module_path.clone())
        .expect("HelloStarknet should be present in the test fixtures");

    let class_hash = declare(&mut cached_state, &module_path, &contracts_data)
        .unwrap()
        .unwrap_success();
    let expected_class_hash = &contracts_data
        .resolve_contract(&module_path)
        .unwrap()
        .class_hash;

    assert_eq!(class_hash, *expected_class_hash);
}

#[test]
fn declare_ambiguous_name_resolved_by_module_path() {
    let contract_name = "HelloStarknet";
    let duplicate_module_path = "duplicate::HelloStarknet".to_string();

    let mut cached_state = create_cached_state();

    // Introduce a second contract sharing the name `HelloStarknet` under a distinct module path,
    // making the bare name ambiguous but each module path still unique.
    let mut contracts_data = get_contracts();
    let (original_module_path, existing) = contracts_data
        .contracts
        .iter()
        .find(|(module_path, _)| contract_name_from_module_path(module_path) == contract_name)
        .map(|(module_path, contract)| (module_path.clone(), contract.clone()))
        .expect("HelloStarknet should be present in the test fixtures");
    contracts_data
        .contracts
        .insert(duplicate_module_path.clone(), existing);

    // The bare name is ambiguous, but passing a full module path resolves it.
    let class_hash = declare(&mut cached_state, &original_module_path, &contracts_data)
        .unwrap()
        .unwrap_success();
    let expected_class_hash = &contracts_data
        .resolve_contract(&original_module_path)
        .unwrap()
        .class_hash;

    assert_eq!(class_hash, *expected_class_hash);
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
        .iter()
        .find(|(module_path, _)| contract_name_from_module_path(module_path) == contract_name)
        .map(|(_, contract)| contract.clone())
        .expect("HelloStarknet should be present in the test fixtures");
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
