use crate::common::assertions::ClassHashAssert;
use crate::common::{get_contracts, state::create_cached_state};
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::{
    declare, DeclareResult,
};
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::CheatcodeError;
use runtime::EnhancedHintError;

#[test]
fn declare_simple() {
    let contract_name = "HelloStarknet";

    let mut cached_state = create_cached_state();

    let contracts_data = get_contracts();

    let class_hash = declare(&mut cached_state, contract_name, &contracts_data)
        .unwrap()
        .unwrap_success();
    let expected_class_hash = contracts_data.get_class_hash(contract_name).unwrap();

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
        let expected_class_hash = contracts_data.get_class_hash(contract_name).unwrap();
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
    let expected_class_hash = contracts_data.get_class_hash(contract_name).unwrap();
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
