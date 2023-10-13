use crate::common::assertions::assert_outputs;
use crate::{
    assert_success,
    common::{
        deploy_contract, felt_selector_from_name, get_contracts, recover_data,
        state::{create_cached_state, create_cheatnet_state},
    },
};
use cairo_felt::Felt252;
use cheatnet::cheatcodes::deploy::deploy;
use cheatnet::rpc::call_contract;
use conversions::StarknetConversions;
use starknet_api::core::ContractAddress;

#[test]
fn prank_simple() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "PrankChecker",
        &[],
    );

    cheatnet_state.start_prank(contract_address, ContractAddress::from(123_u128));

    let selector = felt_selector_from_name("get_caller_address");

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
fn prank_with_other_syscall() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "PrankChecker",
        &[],
    );

    cheatnet_state.start_prank(contract_address, ContractAddress::from(123_u128));

    let selector = felt_selector_from_name("get_caller_address_and_emit_event");

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
fn prank_in_constructor() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contracts = get_contracts();

    let contract_name = "ConstructorPrankChecker".to_owned().to_felt252();
    let class_hash = blockifier_state
        .declare(&contract_name, &contracts)
        .unwrap();
    let precalculated_address = cheatnet_state.precalculate_address(&class_hash, &[]);

    cheatnet_state.start_prank(precalculated_address, ContractAddress::from(123_u128));

    let contract_address = deploy(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[])
        .unwrap()
        .contract_address;

    assert_eq!(precalculated_address, contract_address);

    let selector = felt_selector_from_name("get_stored_caller_address");

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
fn prank_stop() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "PrankChecker",
        &[],
    );

    let selector = felt_selector_from_name("get_caller_address");

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

    let old_address = recover_data(output);

    cheatnet_state.start_prank(contract_address, ContractAddress::from(123_u128));

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

    let new_address = recover_data(output);
    assert_eq!(new_address, vec![Felt252::from(123)]);
    assert_ne!(old_address, new_address);

    cheatnet_state.stop_prank(contract_address);

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();
    let changed_back_address = recover_data(output);

    assert_eq!(old_address, changed_back_address);
}

#[test]
fn prank_double() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "PrankChecker",
        &[],
    );

    let selector = felt_selector_from_name("get_caller_address");

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

    let old_address = recover_data(output);

    cheatnet_state.start_prank(contract_address, ContractAddress::from(123_u128));
    cheatnet_state.start_prank(contract_address, ContractAddress::from(123_u128));

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

    let new_address = recover_data(output);
    assert_eq!(new_address, vec![Felt252::from(123)]);
    assert_ne!(old_address, new_address);

    cheatnet_state.stop_prank(contract_address);

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();
    let changed_back_address = recover_data(output);

    assert_eq!(old_address, changed_back_address);
}

#[test]
fn prank_proxy() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "PrankChecker",
        &[],
    );

    let proxy_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "PrankCheckerProxy",
        &[],
    );

    let proxy_selector = felt_selector_from_name("get_prank_checkers_caller_address");
    let before_prank_output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &proxy_address,
        &proxy_selector,
        &[contract_address.to_felt252()],
    )
    .unwrap();

    cheatnet_state.start_prank(contract_address, ContractAddress::from(123_u128));

    let after_prank_output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &proxy_address,
        &proxy_selector,
        &[contract_address.to_felt252()],
    )
    .unwrap();

    assert_success!(after_prank_output, vec![Felt252::from(123)]);

    cheatnet_state.stop_prank(contract_address);

    let after_prank_cancellation_output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &proxy_address,
        &proxy_selector,
        &[contract_address.to_felt252()],
    )
    .unwrap();

    assert_outputs(before_prank_output, after_prank_cancellation_output);
}

#[test]
fn prank_library_call() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contracts = get_contracts();
    let contract_name = "PrankChecker".to_owned().to_felt252();
    let class_hash = blockifier_state
        .declare(&contract_name, &contracts)
        .unwrap();

    let lib_call_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "PrankCheckerLibCall",
        &[],
    );

    let lib_call_selector = felt_selector_from_name("get_caller_address_with_lib_call");
    let before_prank_output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &lib_call_address,
        &lib_call_selector,
        &[class_hash.to_felt252()],
    )
    .unwrap();

    cheatnet_state.start_prank(lib_call_address, ContractAddress::from(123_u128));

    let after_prank_output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &lib_call_address,
        &lib_call_selector,
        &[class_hash.to_felt252()],
    )
    .unwrap();

    assert_success!(after_prank_output, vec![Felt252::from(123)]);

    cheatnet_state.stop_prank(lib_call_address);

    let after_prank_cancellation_output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &lib_call_address,
        &lib_call_selector,
        &[class_hash.to_felt252()],
    )
    .unwrap();

    assert_outputs(before_prank_output, after_prank_cancellation_output);
}
