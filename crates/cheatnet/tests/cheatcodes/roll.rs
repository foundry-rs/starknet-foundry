use crate::{
    assert_success,
    common::{
        deploy_contract, felt_selector_from_name, get_contracts, recover_data,
        state::create_cheatnet_state,
    },
};
use cairo_felt::Felt252;
use cheatnet::rpc::call_contract;
use conversions::StarknetConversions;

#[test]
fn roll_simple() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "RollChecker", &[]);

    state.start_roll(contract_address, Felt252::from(123_u128));

    let selector = felt_selector_from_name("get_block_number");

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn roll_with_other_syscall() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "RollChecker", &[]);

    state.start_roll(contract_address, Felt252::from(123_u128));

    let selector = felt_selector_from_name("get_block_number_and_emit_event");

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn roll_in_constructor() {
    let mut state = create_cheatnet_state();

    let contracts = get_contracts();

    let contract_name = "ConstructorRollChecker".to_owned().to_felt252();
    let class_hash = state.declare(&contract_name, &contracts).unwrap();
    let precalculated_address = state.precalculate_address(&class_hash, &[]);

    state.start_roll(precalculated_address, Felt252::from(123_u128));

    let contract_address = state.deploy(&class_hash, &[]).unwrap();

    assert_eq!(precalculated_address, contract_address);

    let selector = felt_selector_from_name("get_stored_block_number");

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn roll_stop() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "RollChecker", &[]);

    let selector = felt_selector_from_name("get_block_number");

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    let old_block_number = recover_data(output);

    state.start_roll(contract_address, Felt252::from(123_u128));

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    let new_block_number = recover_data(output);
    assert_eq!(new_block_number, vec![Felt252::from(123)]);
    assert_ne!(old_block_number, new_block_number);

    state.stop_roll(contract_address);

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();
    let changed_back_block_number = recover_data(output);

    assert_eq!(old_block_number, changed_back_block_number);
}

#[test]
fn roll_double() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "RollChecker", &[]);

    let selector = felt_selector_from_name("get_block_number");

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    let old_block_number = recover_data(output);

    state.start_roll(contract_address, Felt252::from(123_u128));
    state.start_roll(contract_address, Felt252::from(123_u128));

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    let new_block_number = recover_data(output);
    assert_eq!(new_block_number, vec![Felt252::from(123)]);
    assert_ne!(old_block_number, new_block_number);

    state.stop_roll(contract_address);

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();
    let changed_back_block_number = recover_data(output);

    assert_eq!(old_block_number, changed_back_block_number);
}

#[test]
fn roll_proxy() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "RollChecker", &[]);

    state.start_roll(contract_address, Felt252::from(123_u128));

    let selector = felt_selector_from_name("get_block_number");

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    assert_success!(output, vec![Felt252::from(123)]);

    let proxy_address = deploy_contract(&mut state, "RollCheckerProxy", &[]);
    let proxy_selector = felt_selector_from_name("get_roll_checkers_block_number");
    let output = call_contract(
        &proxy_address,
        &proxy_selector,
        &[contract_address.to_felt252()],
        &mut state,
    )
    .unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn roll_library_call() {
    let mut state = create_cheatnet_state();

    let contracts = get_contracts();
    let contract_name = "RollChecker".to_owned().to_felt252();
    let class_hash = state.declare(&contract_name, &contracts).unwrap();

    let lib_call_address = deploy_contract(&mut state, "RollCheckerLibCall", &[]);

    state.start_roll(lib_call_address, Felt252::from(123_u128));

    let lib_call_selector = felt_selector_from_name("get_block_number_with_lib_call");
    let output = call_contract(
        &lib_call_address,
        &lib_call_selector,
        &[class_hash.to_felt252()],
        &mut state,
    )
    .unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}
