use crate::assert_success;
use crate::common::state::{create_cached_state, create_cheatnet_state};
use crate::common::{call_contract, deploy_contract, felt_selector_from_name, get_contracts};
use cairo_felt::Felt252;
use cairo_vm::vm::errors::hint_errors::HintError;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    CallFailure, CallResult,
};
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::deploy::{
    deploy, deploy_at,
};
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::CheatcodeError;
use conversions::felt252::FromShortString;
use conversions::IntoConv;
use runtime::EnhancedHintError;
use starknet_api::core::ContractAddress;

#[test]
fn deploy_at_predefined_address() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract = Felt252::from_short_string("HelloStarknet").unwrap();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();
    let contract_address = deploy_at(
        &mut blockifier_state,
        &mut cheatnet_state,
        &class_hash,
        &[],
        ContractAddress::from(1_u8),
    )
    .unwrap()
    .contract_address;

    assert_eq!(contract_address, ContractAddress::from(1_u8));

    let selector = felt_selector_from_name("get_balance");
    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

    assert_success!(output, vec![Felt252::from(0)]);
}

#[test]
fn deploy_two_at_the_same_address() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract = Felt252::from_short_string("HelloStarknet").unwrap();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();
    deploy_at(
        &mut blockifier_state,
        &mut cheatnet_state,
        &class_hash,
        &[],
        ContractAddress::from(1_u8),
    )
    .unwrap();

    let result = deploy_at(
        &mut blockifier_state,
        &mut cheatnet_state,
        &class_hash,
        &[],
        ContractAddress::from(1_u8),
    );

    assert!(matches!(
        result,
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Hint(HintError::CustomHint(err))))
        if err.as_ref() == "Address is already taken"
    ));
}

#[test]
fn call_predefined_contract_from_proxy_contract() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract = Felt252::from_short_string("PrankChecker").unwrap();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();
    let prank_checker_address = deploy_at(
        &mut blockifier_state,
        &mut cheatnet_state,
        &class_hash,
        &[],
        ContractAddress::from(1_u8),
    )
    .unwrap()
    .contract_address;

    assert_eq!(prank_checker_address, ContractAddress::from(1_u8));

    let proxy_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "PrankCheckerProxy",
        &[],
    );
    let proxy_selector = felt_selector_from_name("get_prank_checkers_caller_address");
    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &proxy_address,
        &proxy_selector,
        &[prank_checker_address.into_()],
    )
    .unwrap();

    assert_success!(output, vec![proxy_address.into_()]);
}

#[test]
fn deploy_contract_on_predefined_address_after_its_usage() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let proxy_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "SpyEventsCheckerProxy",
        &[Felt252::from(121)],
    );

    let proxy_selector = felt_selector_from_name("emit_one_event");
    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &proxy_address,
        &proxy_selector,
        &[Felt252::from(323)],
    )
    .unwrap();

    assert!(
        matches!(
            output.result,
            CallResult::Failure(CallFailure::Error { msg, .. })
            if msg.contains("Requested contract address") && msg.contains("is not deployed")
        ),
        "Wrong error message"
    );

    let contract = Felt252::from_short_string("SpyEventsChecker").unwrap();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();
    deploy_at(
        &mut blockifier_state,
        &mut cheatnet_state,
        &class_hash,
        &[],
        ContractAddress::from(121_u8),
    )
    .unwrap();

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &proxy_address,
        &proxy_selector,
        &[Felt252::from(323)],
    )
    .unwrap();

    assert_success!(output, vec![]);
}

#[test]
fn try_to_deploy_at_0() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract = Felt252::from_short_string("HelloStarknet").unwrap();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();
    let output = deploy_at(
        &mut blockifier_state,
        &mut cheatnet_state,
        &class_hash,
        &[],
        ContractAddress::from(0_u8),
    );

    assert!(match output {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Hint(HintError::CustomHint(msg)))) =>
            msg.as_ref() == "Cannot deploy contract at address 0.",
        _ => false,
    });
}

#[test]
fn deploy_calldata_no_constructor() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract = Felt252::from_short_string("HelloStarknet").unwrap();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();

    let output = deploy(
        &mut blockifier_state,
        &mut cheatnet_state,
        &class_hash,
        &[Felt252::from(123_321)],
    );

    assert!(match output {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Hint(HintError::CustomHint(msg)))) =>
            msg.as_ref()
                .contains("Cannot pass calldata to a contract with no constructor"),
        _ => false,
    });
}

#[test]
fn deploy_missing_arguments_in_constructor() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract = Felt252::from_short_string("ConstructorSimple2").unwrap();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();

    let output = deploy(
        &mut blockifier_state,
        &mut cheatnet_state,
        &class_hash,
        &[Felt252::from(123_321)],
    );

    assert!(match output {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Hint(HintError::CustomHint(msg)))) =>
            msg.as_ref() == "0x4661696c656420746f20646573657269616c697a6520706172616d202332 ('Failed to deserialize param #2')",
        _ => false,
    });
}

#[test]
fn deploy_too_many_arguments_in_constructor() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract = Felt252::from_short_string("ConstructorSimple").unwrap();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();

    let output = deploy(
        &mut blockifier_state,
        &mut cheatnet_state,
        &class_hash,
        &[Felt252::from(123_321), Felt252::from(523_325)],
    );

    assert!(match output {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Hint(HintError::CustomHint(msg)))) =>
            msg.as_ref() == "0x496e70757420746f6f206c6f6e6720666f7220617267756d656e7473 ('Input too long for arguments')",
        _ => false,
    });
}

#[test]
fn deploy_invalid_class_hash() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let class_hash = Felt252::from_short_string("Invalid ClassHash")
        .unwrap()
        .into_();

    let output = deploy(
        &mut blockifier_state,
        &mut cheatnet_state,
        &class_hash,
        &[Felt252::from(123_321), Felt252::from(523_325)],
    );

    assert!(matches!(
        output,
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Hint(HintError::CustomHint(msg))))
        if msg.as_ref().contains(class_hash.to_string().as_str()),
    ));
}

#[test]
fn deploy_invokes_constructor() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "ConstructorSimple",
        &[Felt252::from(123)],
    );

    let selector = felt_selector_from_name("get_number");

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn deploy_at_invokes_constructor() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract = Felt252::from_short_string("ConstructorSimple").unwrap();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();

    let contract_address = deploy_at(
        &mut blockifier_state,
        &mut cheatnet_state,
        &class_hash,
        &[Felt252::from(123)],
        Felt252::from(420).into_(),
    )
    .unwrap()
    .contract_address;

    let selector = felt_selector_from_name("get_number");

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}
