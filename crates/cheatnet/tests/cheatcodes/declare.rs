use crate::common::assertions::ClassHashAssert;
use crate::common::{get_contracts, state::create_cached_state};
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::CheatcodeError;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::{
    DeclareResult, declare, declare_from_file,
};
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use runtime::EnhancedHintError;
use shared::utils::contract_name_from_module_path;
use starknet_api::core::ClassHash;
use starknet_types_core::felt::Felt;
use std::path::Path;

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
        .find(|(module_path, _)| contract_name_from_module_path(module_path) == contract_name)
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
fn declare_ambiguous_name_resolved_by_partial_module_path() {
    let contract_name = "HelloStarknet";
    let nested_module_path = "pkg::nested::a::HelloStarknet".to_string();
    let other_module_path = "pkg::a::HelloStarknet".to_string();

    let mut cached_state = create_cached_state();

    let mut contracts_data = get_contracts();
    let existing = contracts_data
        .contracts
        .iter()
        .find(|(module_path, _)| contract_name_from_module_path(module_path) == contract_name)
        .map(|(_module_path, contract)| contract.clone())
        .expect("HelloStarknet should be present in the test fixtures");
    contracts_data
        .contracts
        .insert(nested_module_path.clone(), existing.clone());
    contracts_data.contracts.insert(other_module_path, existing);

    let partial_module_path = "nested::a::HelloStarknet";
    let class_hash = declare(&mut cached_state, partial_module_path, &contracts_data)
        .unwrap()
        .unwrap_success();
    let expected_class_hash = &contracts_data
        .resolve_contract(&nested_module_path)
        .unwrap()
        .class_hash;

    assert_eq!(class_hash, *expected_class_hash);
}

#[test]
fn declare_ambiguous_name_resolved_by_partial_module_path_with_leading_colons() {
    let contract_name = "HelloStarknet";
    let module_path = "pkg::a::HelloStarknet".to_string();

    let mut cached_state = create_cached_state();

    let mut contracts_data = get_contracts();
    let existing = contracts_data
        .contracts
        .iter()
        .find(|(module_path, _)| contract_name_from_module_path(module_path) == contract_name)
        .map(|(_module_path, contract)| contract.clone())
        .expect("HelloStarknet should be present in the test fixtures");
    contracts_data
        .contracts
        .insert(module_path.clone(), existing);

    let class_hash = declare(&mut cached_state, "::a::HelloStarknet", &contracts_data)
        .unwrap()
        .unwrap_success();
    let expected_class_hash = &contracts_data
        .resolve_contract(&module_path)
        .unwrap()
        .class_hash;

    assert_eq!(class_hash, *expected_class_hash);
}

#[test]
fn declare_exact_module_path_takes_precedence_over_suffix_match() {
    let contract_name = "HelloStarknet";
    let partial_module_path = "pkg::a::HelloStarknet".to_string();
    let full_module_path = "outer::pkg::a::HelloStarknet".to_string();
    let partial_class_hash = ClassHash(Felt::from(123));
    let long_class_hash = ClassHash(Felt::from(456));

    let mut cached_state = create_cached_state();

    let mut contracts_data = get_contracts();
    let existing = contracts_data
        .contracts
        .iter()
        .find(|(module_path, _)| contract_name_from_module_path(module_path) == contract_name)
        .map(|(_module_path, contract)| contract.clone())
        .expect("HelloStarknet should be present in the test fixtures");

    let mut partial_contract = existing.clone();
    partial_contract.class_hash = partial_class_hash;
    let mut full_contract = existing;
    full_contract.class_hash = long_class_hash;

    contracts_data
        .contracts
        .insert(partial_module_path.clone(), partial_contract);
    contracts_data
        .contracts
        .insert(full_module_path.clone(), full_contract);

    let class_hash = declare(&mut cached_state, &full_module_path, &contracts_data)
        .unwrap()
        .unwrap_success();
    assert_eq!(class_hash, long_class_hash);

    let class_hash = declare(&mut cached_state, &partial_module_path, &contracts_data)
        .unwrap()
        .unwrap_success();
    assert_eq!(class_hash, partial_class_hash);

    let output = declare(
        &mut cached_state,
        &format!("::{partial_module_path}"),
        &contracts_data,
    )
    .unwrap();
    assert!(
        matches!(output, DeclareResult::AlreadyDeclared(class_hash) if class_hash == partial_class_hash)
    );
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
        .map(|(_module_path, contract)| contract.clone())
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

#[test]
fn declare_ambiguous_partial_module_path() {
    let contract_name = "HelloStarknet";
    let nested_module_path = "pkg::nested::a::HelloStarknet".to_string();
    let other_module_path = "pkg::a::HelloStarknet".to_string();

    let mut cached_state = create_cached_state();

    let mut contracts_data = get_contracts();
    let existing = contracts_data
        .contracts
        .iter()
        .find(|(module_path, _)| contract_name_from_module_path(module_path) == contract_name)
        .map(|(_module_path, contract)| contract.clone())
        .expect("HelloStarknet should be present in the test fixtures");
    contracts_data
        .contracts
        .insert(nested_module_path.clone(), existing.clone());
    contracts_data
        .contracts
        .insert(other_module_path.clone(), existing);

    let output = declare(&mut cached_state, "a::HelloStarknet", &contracts_data);

    assert!(match output {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Anyhow(msg))) => {
            let msg = msg.to_string();
            msg.contains("Multiple contracts found")
                && msg.contains("a::HelloStarknet")
                && msg.contains(&nested_module_path)
                && msg.contains(&other_module_path)
        }
        _ => false,
    });
}

#[test]
fn declare_from_file_simple() {
    let contract_name = "HelloStarknet";
    let sierra_path = Path::new(
        "tests/contracts/target/dev/cheatnet_testing_contracts_HelloStarknet.contract_class.json",
    );

    let mut cached_state = create_cached_state();

    let contracts_data = get_contracts();

    let class_hash = declare_from_file(&mut cached_state, sierra_path, &contracts_data)
        .unwrap()
        .unwrap_success();
    let expected_class_hash = &contracts_data
        .resolve_contract(contract_name)
        .unwrap()
        .class_hash;

    assert_eq!(class_hash, *expected_class_hash);

    let output = declare_from_file(&mut cached_state, sierra_path, &contracts_data);

    assert!(
        matches!(output, Ok(DeclareResult::AlreadyDeclared(class_hash)) if class_hash == *expected_class_hash)
    );
}

#[test]
fn declare_from_file_nonexistent_path() {
    let sierra_path = Path::new("non_existent.contract_class.json");

    let mut cached_state = create_cached_state();

    let output = declare_from_file(&mut cached_state, sierra_path, &ContractsData::default());

    assert!(match output {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Anyhow(msg))) => {
            let msg = msg.to_string();
            msg.contains("Failed to read Sierra file")
                && msg.contains(sierra_path.to_str().unwrap())
        }
        _ => false,
    });
}

#[test]
fn declare_from_file_invalid_file() {
    let sierra_path = Path::new("tests/data/invalid_contract_class.json");

    let mut cached_state = create_cached_state();

    let output = declare_from_file(&mut cached_state, sierra_path, &ContractsData::default());

    assert!(match output {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Anyhow(msg))) => {
            let msg = msg.to_string();
            msg.contains("Failed to parse Sierra contract class JSON")
                && msg.contains(sierra_path.to_str().unwrap())
        }
        _ => false,
    });
}
