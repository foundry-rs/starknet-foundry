use crate::assert_success;
use crate::common::state::create_cheatnet_state;
use crate::common::{deploy_contract, felt_selector_from_name, get_contracts};
use blockifier::state::errors::StateError::UndeclaredClassHash;
use cairo_felt::Felt252;
use cairo_vm::vm::errors::hint_errors::HintError;
use cheatnet::cheatcodes::{CheatcodeError, EnhancedHintError};
use cheatnet::rpc::{call_contract, CallContractOutput};
use conversions::StarknetConversions;
use starknet_api::core::ContractAddress;

#[test]
fn deploy_at_predefined_address() {
    let mut state = create_cheatnet_state();

    let contract = "HelloStarknet".to_owned().to_felt252();
    let contracts = get_contracts();

    let class_hash = state.declare(&contract, &contracts).unwrap();
    let contract_address = state
        .deploy_at(&class_hash, &[], ContractAddress::from(1_u8))
        .unwrap()
        .contract_address;

    assert_eq!(contract_address, ContractAddress::from(1_u8));

    let selector = felt_selector_from_name("get_balance");
    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    assert_success!(output, vec![Felt252::from(0)]);
}

#[test]
fn deploy_two_at_the_same_address() {
    let mut state = create_cheatnet_state();

    let contract = "HelloStarknet".to_owned().to_felt252();
    let contracts = get_contracts();

    let class_hash = state.declare(&contract, &contracts).unwrap();
    state
        .deploy_at(&class_hash, &[], ContractAddress::from(1_u8))
        .unwrap();

    let result = state.deploy_at(&class_hash, &[], ContractAddress::from(1_u8));

    assert!(match result {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Hint(HintError::CustomHint(err)))) =>
            err.as_ref() == "Address is already taken",
        _ => false,
    });
}

#[test]
fn call_predefined_contract_from_proxy_contract() {
    let mut state = create_cheatnet_state();

    let contract = "PrankChecker".to_owned().to_felt252();
    let contracts = get_contracts();

    let class_hash = state.declare(&contract, &contracts).unwrap();
    let prank_checker_address = state
        .deploy_at(&class_hash, &[], ContractAddress::from(1_u8))
        .unwrap()
        .contract_address;

    assert_eq!(prank_checker_address, ContractAddress::from(1_u8));

    let proxy_address = deploy_contract(&mut state, "PrankCheckerProxy", &[]);
    let proxy_selector = felt_selector_from_name("get_prank_checkers_caller_address");
    let output = call_contract(
        &proxy_address,
        &proxy_selector,
        &[prank_checker_address.to_felt252()],
        &mut state,
    )
    .unwrap();

    assert_success!(output, vec![proxy_address.to_felt252()]);
}

#[test]
fn deploy_contract_on_predefined_address_after_its_usage() {
    let mut state = create_cheatnet_state();

    let proxy_address = deploy_contract(&mut state, "SpyEventsCheckerProxy", &[Felt252::from(121)]);

    let proxy_selector = felt_selector_from_name("emit_one_event");
    let output = call_contract(
        &proxy_address,
        &proxy_selector,
        &[Felt252::from(323)],
        &mut state,
    )
    .unwrap();

    assert!(match output {
        CallContractOutput::Error { msg, .. } =>
            msg.contains("Requested contract address") && msg.contains("is not deployed"),
        _ => false,
    });

    let contract = "SpyEventsChecker".to_owned().to_felt252();
    let contracts = get_contracts();

    let class_hash = state.declare(&contract, &contracts).unwrap();
    state
        .deploy_at(&class_hash, &[], ContractAddress::from(121_u8))
        .unwrap();

    let output = call_contract(
        &proxy_address,
        &proxy_selector,
        &[Felt252::from(323)],
        &mut state,
    )
    .unwrap();

    assert_success!(output, vec![]);
}

#[test]
fn try_to_deploy_at_0() {
    let mut state = create_cheatnet_state();

    let contract = "HelloStarknet".to_owned().to_felt252();
    let contracts = get_contracts();

    let class_hash = state.declare(&contract, &contracts).unwrap();
    let output = state.deploy_at(&class_hash, &[], ContractAddress::from(0_u8));

    assert!(match output {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Hint(HintError::CustomHint(msg)))) =>
            msg.as_ref() == "Cannot deploy contract at address 0.",
        _ => false,
    });
}

#[test]
fn deploy_calldata_no_constructor() {
    let mut state = create_cheatnet_state();

    let contract = "HelloStarknet".to_owned().to_felt252();
    let contracts = get_contracts();

    let class_hash = state.declare(&contract, &contracts).unwrap();

    let output = state.deploy(&class_hash, &[Felt252::from(123_321)]);

    assert!(match output {
        Err(CheatcodeError::Recoverable(data)) =>
            data[0] == "No constructor in contract".to_owned().to_felt252(),
        _ => false,
    });
}

#[test]
fn deploy_missing_arguments_in_constructor() {
    let mut state = create_cheatnet_state();

    let contract = "ConstructorSimple2".to_owned().to_felt252();
    let contracts = get_contracts();

    let class_hash = state.declare(&contract, &contracts).unwrap();

    let output = state.deploy(&class_hash, &[Felt252::from(123_321)]);

    assert!(match output {
        Err(CheatcodeError::Recoverable(data)) =>
            data[0] == "Failed to deserialize param #2".to_owned().to_felt252(),
        _ => false,
    });
}

#[test]
fn deploy_too_many_arguments_in_constructor() {
    let mut state = create_cheatnet_state();

    let contract = "ConstructorSimple".to_owned().to_felt252();
    let contracts = get_contracts();

    let class_hash = state.declare(&contract, &contracts).unwrap();

    let output = state.deploy(
        &class_hash,
        &[Felt252::from(123_321), Felt252::from(523_325)],
    );

    assert!(match output {
        Err(CheatcodeError::Recoverable(data)) =>
            data[0] == "Input too long for arguments".to_owned().to_felt252(),
        _ => false,
    });
}

#[test]
fn deploy_invalid_class_hash() {
    let mut state = create_cheatnet_state();

    let class_hash = "Invalid ClassHash".to_owned().to_class_hash();

    let output = state.deploy(
        &class_hash,
        &[Felt252::from(123_321), Felt252::from(523_325)],
    );

    assert!(match output {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::State(UndeclaredClassHash(
            class_hash2,
        )))) => class_hash == class_hash2,
        _ => false,
    });
}

#[test]
fn deploy_invokes_constructor() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "ConstructorSimple", &[Felt252::from(123)]);

    let selector = felt_selector_from_name("get_number");

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}
