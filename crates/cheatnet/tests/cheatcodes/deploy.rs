use crate::assert_success;
use crate::common::state::create_cheatnet_state;
use crate::common::{deploy_contract, felt_selector_from_name, get_contracts};
use cairo_felt::Felt252;
use cairo_vm::vm::errors::hint_errors::HintError;
use cheatnet::cheatcodes::deploy::{deploy_at, deploy};
use cheatnet::cheatcodes::{CheatcodeError, EnhancedHintError};
use cheatnet::rpc::{call_contract, CallContractOutput};
use conversions::StarknetConversions;
use starknet_api::core::ContractAddress;

#[test]
fn deploy_at_predefined_address() {
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state();

    let contract = "HelloStarknet".to_owned().to_felt252();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();
    let contract_address = deploy_at(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[], ContractAddress::from(1_u8))
        .unwrap()
        .contract_address;

    assert_eq!(contract_address, ContractAddress::from(1_u8));

    let selector = felt_selector_from_name("get_balance");
    let output = call_contract(&mut blockifier_state,  &mut cheatnet_state, &contract_address, &selector, &[]).unwrap();

    assert_success!(output, vec![Felt252::from(0)]);
}

#[test]
fn deploy_two_at_the_same_address() {
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state();

    let contract = "HelloStarknet".to_owned().to_felt252();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();
    deploy_at(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[], ContractAddress::from(1_u8)).unwrap();

    let result = deploy_at(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[], ContractAddress::from(1_u8));

    assert!(match result {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Hint(HintError::CustomHint(err)))) =>
            err.as_ref() == "Address is already taken",
        _ => false,
    });
}

#[test]
fn call_predefined_contract_from_proxy_contract() {
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state();

    let contract = "PrankChecker".to_owned().to_felt252();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();
    let prank_checker_address = deploy_at(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[], ContractAddress::from(1_u8))
        .unwrap()
        .contract_address;

    assert_eq!(prank_checker_address, ContractAddress::from(1_u8));

    let proxy_address = deploy_contract(&mut blockifier_state, &mut cheatnet_state, "PrankCheckerProxy", &[]);
    let proxy_selector = felt_selector_from_name("get_prank_checkers_caller_address");
    let output = call_contract(
        &mut blockifier_state, &mut cheatnet_state,
        &proxy_address,
        &proxy_selector,
        &[prank_checker_address.to_felt252()],
    )
    .unwrap();

    assert_success!(output, vec![proxy_address.to_felt252()]);
}

#[test]
fn deploy_contract_on_predefined_address_after_its_usage() {
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state();

    let proxy_address = deploy_contract(&mut blockifier_state, &mut cheatnet_state, "SpyEventsCheckerProxy", &[Felt252::from(121)]);

    let proxy_selector = felt_selector_from_name("emit_one_event");
    let output = call_contract(
        &mut blockifier_state, &mut cheatnet_state,
        &proxy_address,
        &proxy_selector,
        &[Felt252::from(323)],
    )
    .unwrap();

    assert!(match output {
        CallContractOutput::Error { msg, .. } =>
            msg.contains("Requested contract address") && msg.contains("is not deployed"),
        _ => false,
    });

    let contract = "SpyEventsChecker".to_owned().to_felt252();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();
    deploy_at(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[], ContractAddress::from(121_u8)) .unwrap();

    let output = call_contract(
        &mut blockifier_state, &mut cheatnet_state,
        &proxy_address,
        &proxy_selector,
        &[Felt252::from(323)],
    )
    .unwrap();

    assert_success!(output, vec![]);
}

#[test]
fn try_to_deploy_at_0() {
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state();

    let contract = "HelloStarknet".to_owned().to_felt252();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();
    let output = deploy_at(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[], ContractAddress::from(0_u8));

    assert!(match output {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Hint(HintError::CustomHint(msg)))) =>
            msg.as_ref() == "Cannot deploy contract at address 0.",
        _ => false,
    });
}

#[test]
fn deploy_calldata_no_constructor() {
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state();

    let contract = "HelloStarknet".to_owned().to_felt252();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();

    let output = deploy(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[Felt252::from(123_321)]);

    assert!(match output {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Hint(HintError::CustomHint(msg)))) =>
            msg.as_ref()
                .contains("Cannot pass calldata to a contract with no constructor"),
        _ => false,
    });
}

#[test]
fn deploy_missing_arguments_in_constructor() {
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state();

    let contract = "ConstructorSimple2".to_owned().to_felt252();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();

    let output = deploy(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[Felt252::from(123_321)]);

    assert!(match output {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Hint(HintError::CustomHint(msg)))) =>
            msg.as_ref() == "Failed to deserialize param #2",
        _ => false,
    });
}

#[test]
fn deploy_too_many_arguments_in_constructor() {
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state();

    let contract = "ConstructorSimple".to_owned().to_felt252();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();

    let output = deploy(
        &mut blockifier_state, &mut cheatnet_state,
        &class_hash,
        &[Felt252::from(123_321), Felt252::from(523_325)],
    );

    assert!(match output {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Hint(HintError::CustomHint(msg)))) =>
            msg.as_ref() == "Input too long for arguments",
        _ => false,
    });
}

#[test]
fn deploy_invalid_class_hash() {
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state();

    let class_hash = "Invalid ClassHash".to_owned().to_class_hash();

    let output = deploy(
        &mut blockifier_state, &mut cheatnet_state,
        &class_hash,
        &[Felt252::from(123_321), Felt252::from(523_325)],
    );

    assert!(match output {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Hint(HintError::CustomHint(msg)))) =>
            msg.as_ref().contains(class_hash.to_string().as_str()),
        _ => false,
    });
}

#[test]
fn deploy_invokes_constructor() {
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state();

    let contract_address = deploy_contract(&mut blockifier_state, &mut cheatnet_state, "ConstructorSimple", &[Felt252::from(123)]);

    let selector = felt_selector_from_name("get_number");

    let output = call_contract(&mut blockifier_state, &mut cheatnet_state, &contract_address, &selector, &[]).unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}
