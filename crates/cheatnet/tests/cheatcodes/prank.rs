use crate::{
    assert_success,
    common::{deploy_contract, get_contracts, recover_data, state::create_cheatnet_state},
};
use cairo_felt::Felt252;
use cheatnet::{
    conversions::{
        class_hash_to_felt, contract_address_to_felt, felt_from_short_string,
        felt_selector_from_name,
    },
    rpc::call_contract,
};
use starknet_api::core::ContractAddress;

#[test]
fn prank_simple() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "PrankChecker", &[]);

    state.start_prank(contract_address, ContractAddress::from(123_u128));

    let selector = felt_selector_from_name("get_caller_address");

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn prank_with_other_syscall() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "PrankChecker", &[]);

    state.start_prank(contract_address, ContractAddress::from(123_u128));

    let selector = felt_selector_from_name("get_caller_address_and_emit_event");

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
#[ignore = "TODO(#254)"]
fn prank_in_constructor() {
    let mut state = create_cheatnet_state();

    let contracts = get_contracts();

    let contract_name = felt_from_short_string("ConstructorPrankChecker");
    let class_hash = state.declare(&contract_name, &contracts).unwrap();
    let precalculated_address = state.precalculate_address(&class_hash, &[]);

    state.start_prank(precalculated_address, ContractAddress::from(123_u128));

    let contract_address = state.deploy(&class_hash, &[]).unwrap();

    assert_eq!(precalculated_address, contract_address);

    let selector = felt_selector_from_name("get_stored_caller_address");

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn prank_stop() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "PrankChecker", &[]);

    let selector = felt_selector_from_name("get_caller_address");

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    let old_address = recover_data(output);

    state.start_prank(contract_address, ContractAddress::from(123_u128));

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    let new_address = recover_data(output);
    assert_eq!(new_address, vec![Felt252::from(123)]);
    assert_ne!(old_address, new_address);

    state.stop_prank(contract_address);

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();
    let changed_back_address = recover_data(output);

    assert_eq!(old_address, changed_back_address);
}

#[test]
fn prank_double() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "PrankChecker", &[]);

    let selector = felt_selector_from_name("get_caller_address");

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    let old_address = recover_data(output);

    state.start_prank(contract_address, ContractAddress::from(123_u128));
    state.start_prank(contract_address, ContractAddress::from(123_u128));

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    let new_address = recover_data(output);
    assert_eq!(new_address, vec![Felt252::from(123)]);
    assert_ne!(old_address, new_address);

    state.stop_prank(contract_address);

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();
    let changed_back_address = recover_data(output);

    assert_eq!(old_address, changed_back_address);
}

#[test]
fn prank_proxy() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "PrankChecker", &[]);

    state.start_prank(contract_address, ContractAddress::from(123_u128));

    let selector = felt_selector_from_name("get_caller_address");

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    assert_success!(output, vec![Felt252::from(123)]);

    let proxy_address = deploy_contract(&mut state, "PrankCheckerProxy", &[]);
    let proxy_selector = felt_selector_from_name("get_prank_checkers_caller_address");
    let output = call_contract(
        &proxy_address,
        &proxy_selector,
        &[contract_address_to_felt(contract_address)],
        &mut state,
    )
    .unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn prank_library_call() {
    let mut state = create_cheatnet_state();

    let contracts = get_contracts();
    let contract_name = felt_from_short_string("PrankChecker");
    let class_hash = state.declare(&contract_name, &contracts).unwrap();

    let lib_call_address = deploy_contract(&mut state, "PrankCheckerLibCall", &[]);

    state.start_prank(lib_call_address, ContractAddress::from(123_u128));

    let lib_call_selector = felt_selector_from_name("get_caller_address_with_lib_call");
    let output = call_contract(
        &lib_call_address,
        &lib_call_selector,
        &[class_hash_to_felt(class_hash)],
        &mut state,
    )
    .unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}
