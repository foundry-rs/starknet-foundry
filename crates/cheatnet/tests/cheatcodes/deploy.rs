use crate::common::assertions::{ClassHashAssert, assert_success};
use crate::common::state::create_cached_state;
use crate::common::{
    call_contract, deploy_at_wrapper, deploy_contract, deploy_wrapper, felt_selector_from_name,
    get_contracts,
};
use cairo_vm::vm::errors::hint_errors::HintError;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    CallFailure, CallResult,
};
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::CheatcodeError;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::declare;
use cheatnet::state::CheatnetState;
use conversions::IntoConv;
use conversions::felt::FromShortString;
use runtime::EnhancedHintError;
use starknet_api::core::ContractAddress;
use starknet_types_core::felt::Felt;

#[test]
fn deploy_at_predefined_address() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contracts_data = get_contracts();

    let class_hash = declare(&mut cached_state, "HelloStarknet", &contracts_data)
        .unwrap()
        .unwrap_success();
    let contract_address = deploy_at_wrapper(
        &mut cached_state,
        &mut cheatnet_state,
        &class_hash,
        &[],
        ContractAddress::from(1_u8),
    )
    .unwrap();

    assert_eq!(contract_address, ContractAddress::from(1_u8));

    let selector = felt_selector_from_name("get_balance");
    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );

    assert_success(output, &[Felt::from(0)]);
}

#[test]
fn deploy_two_at_the_same_address() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contracts_data = get_contracts();

    let class_hash = declare(&mut cached_state, "HelloStarknet", &contracts_data)
        .unwrap()
        .unwrap_success();
    deploy_at_wrapper(
        &mut cached_state,
        &mut cheatnet_state,
        &class_hash,
        &[],
        ContractAddress::from(1_u8),
    )
    .unwrap();

    let result = deploy_at_wrapper(
        &mut cached_state,
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
    let mut cheatnet_state = CheatnetState::default();

    let contracts_data = get_contracts();

    let class_hash = declare(
        &mut cached_state,
        "CheatCallerAddressChecker",
        &contracts_data,
    )
    .unwrap()
    .unwrap_success();
    let cheat_caller_address_checker_address = deploy_at_wrapper(
        &mut cached_state,
        &mut cheatnet_state,
        &class_hash,
        &[],
        ContractAddress::from(1_u8),
    )
    .unwrap();

    assert_eq!(
        cheat_caller_address_checker_address,
        ContractAddress::from(1_u8)
    );

    let proxy_address = deploy_contract(
        &mut cached_state,
        &mut cheatnet_state,
        "CheatCallerAddressCheckerProxy",
        &[],
    );
    let proxy_selector = felt_selector_from_name("get_cheated_caller_address");
    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &proxy_address,
        proxy_selector,
        &[cheat_caller_address_checker_address.into_()],
    );

    assert_success(output, &[proxy_address.into_()]);
}

#[test]
fn deploy_contract_on_predefined_address_after_its_usage() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let proxy_address = deploy_contract(
        &mut cached_state,
        &mut cheatnet_state,
        "SpyEventsCheckerProxy",
        &[Felt::from(121)],
    );

    let proxy_selector = felt_selector_from_name("emit_one_event");
    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &proxy_address,
        proxy_selector,
        &[Felt::from(323)],
    );

    assert!(
        matches!(
            output,
            CallResult::Failure(CallFailure::Error { msg, .. })
            if msg.to_string().contains("Requested contract address") && msg.to_string().contains("is not deployed")
        ),
        "Wrong error message"
    );

    let contracts_data = get_contracts();

    let class_hash = declare(&mut cached_state, "SpyEventsChecker", &contracts_data)
        .unwrap()
        .unwrap_success();
    deploy_at_wrapper(
        &mut cached_state,
        &mut cheatnet_state,
        &class_hash,
        &[],
        ContractAddress::from(121_u8),
    )
    .unwrap();

    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &proxy_address,
        proxy_selector,
        &[Felt::from(323)],
    );

    assert_success(output, &[]);
}

#[test]
fn try_to_deploy_at_0() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contracts_data = get_contracts();

    let class_hash = declare(&mut cached_state, "HelloStarknet", &contracts_data)
        .unwrap()
        .unwrap_success();
    let output = deploy_at_wrapper(
        &mut cached_state,
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
    let mut cheatnet_state = CheatnetState::default();

    let contracts_data = get_contracts();

    let class_hash = declare(&mut cached_state, "HelloStarknet", &contracts_data)
        .unwrap()
        .unwrap_success();

    let output = deploy_wrapper(
        &mut cached_state,
        &mut cheatnet_state,
        &class_hash,
        &[Felt::from(123_321)],
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
    let mut cheatnet_state = CheatnetState::default();

    let contracts_data = get_contracts();

    let class_hash = declare(&mut cached_state, "ConstructorSimple2", &contracts_data)
        .unwrap()
        .unwrap_success();

    let output = deploy_wrapper(
        &mut cached_state,
        &mut cheatnet_state,
        &class_hash,
        &[Felt::from(123_321)],
    );

    assert!(match output {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Hint(HintError::CustomHint(msg)))) =>
            msg.as_ref()
                == "\n    0x4661696c656420746f20646573657269616c697a6520706172616d202332 ('Failed to deserialize param #2')\n",
        _ => false,
    });
}

#[test]
fn deploy_too_many_arguments_in_constructor() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contracts_data = get_contracts();

    let class_hash = declare(&mut cached_state, "ConstructorSimple", &contracts_data)
        .unwrap()
        .unwrap_success();

    let output = deploy_wrapper(
        &mut cached_state,
        &mut cheatnet_state,
        &class_hash,
        &[Felt::from(123_321), Felt::from(523_325)],
    );

    assert!(match output {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Hint(HintError::CustomHint(msg)))) =>
            msg.as_ref()
                == "\n    0x496e70757420746f6f206c6f6e6720666f7220617267756d656e7473 ('Input too long for arguments')\n",
        _ => false,
    });
}

#[test]
fn deploy_invalid_class_hash() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let class_hash = Felt::from_short_string("Invalid ClassHash")
        .unwrap()
        .into_();

    let output = deploy_wrapper(
        &mut cached_state,
        &mut cheatnet_state,
        &class_hash,
        &[Felt::from(123_321), Felt::from(523_325)],
    );

    assert!(matches!(
        output,
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Hint(HintError::CustomHint(msg))))
        if msg.as_ref().contains(class_hash.to_hex_string().trim_start_matches("0x")),
    ));
}

#[test]
fn deploy_invokes_constructor() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contract_address = deploy_contract(
        &mut cached_state,
        &mut cheatnet_state,
        "ConstructorSimple",
        &[Felt::from(123)],
    );

    let selector = felt_selector_from_name("get_number");

    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );

    assert_success(output, &[Felt::from(123)]);
}

#[test]
fn deploy_at_invokes_constructor() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contracts_data = get_contracts();

    let class_hash = declare(&mut cached_state, "ConstructorSimple", &contracts_data)
        .unwrap()
        .unwrap_success();

    let contract_address = deploy_at_wrapper(
        &mut cached_state,
        &mut cheatnet_state,
        &class_hash,
        &[Felt::from(123)],
        Felt::from(420).into_(),
    )
    .unwrap();

    let selector = felt_selector_from_name("get_number");

    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );

    assert_success(output, &[Felt::from(123)]);
}
