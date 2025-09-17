use crate::common::assertions::{ClassHashAssert, assert_success};
use crate::common::state::create_cached_state;
use crate::common::{call_contract, deploy_at_wrapper, deploy_contract, get_contracts};
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    CallFailure, CallResult,
};
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::declare;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::storage::selector_from_name;
use cheatnet::state::CheatnetState;
use conversions::IntoConv;
use starknet_api::core::ContractAddress;
use starknet_types_core::felt::Felt;

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
    let proxy_selector = selector_from_name("get_cheated_caller_address");
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

    let proxy_selector = selector_from_name("emit_one_event");
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
fn deploy_invokes_constructor() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contract_address = deploy_contract(
        &mut cached_state,
        &mut cheatnet_state,
        "ConstructorSimple",
        &[Felt::from(123)],
    );

    let selector = selector_from_name("get_number");

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

    let selector = selector_from_name("get_number");

    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );

    assert_success(output, &[Felt::from(123)]);
}
