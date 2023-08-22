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

#[test]
fn warp_simple() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "WarpChecker", vec![].as_slice());

    state.start_warp(contract_address, Felt252::from(123_u128));

    let selector = felt_selector_from_name("get_block_timestamp");

    let output =
        call_contract(&contract_address, &selector, vec![].as_slice(), &mut state).unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn warp_with_other_syscall() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "WarpChecker", vec![].as_slice());

    state.start_warp(contract_address, Felt252::from(123_u128));

    let selector = felt_selector_from_name("get_block_timestamp_and_emit_event");

    let output =
        call_contract(&contract_address, &selector, vec![].as_slice(), &mut state).unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
#[ignore = "TODO(#254)"]
fn warp_in_constructor() {
    let mut state = create_cheatnet_state();

    let contracts = get_contracts();

    let contract_name = felt_from_short_string("ConstructorWarpChecker");
    let class_hash = state.declare(&contract_name, &contracts).unwrap();
    let precalculated_address = state.precalculate_address(&class_hash, vec![].as_slice());

    state.start_warp(precalculated_address, Felt252::from(123_u128));

    let contract_address = state.deploy(&class_hash, vec![].as_slice()).unwrap();

    assert_eq!(precalculated_address, contract_address);

    let selector = felt_selector_from_name("get_stored_block_timestamp");

    let output =
        call_contract(&contract_address, &selector, vec![].as_slice(), &mut state).unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn warp_stop() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "WarpChecker", vec![].as_slice());

    let selector = felt_selector_from_name("get_block_timestamp");

    let output =
        call_contract(&contract_address, &selector, vec![].as_slice(), &mut state).unwrap();

    let old_block_timestamp = recover_data(output);

    state.start_warp(contract_address, Felt252::from(123_u128));

    let output =
        call_contract(&contract_address, &selector, vec![].as_slice(), &mut state).unwrap();

    let new_block_timestamp = recover_data(output);
    assert_eq!(new_block_timestamp, vec![Felt252::from(123)]);
    assert_ne!(old_block_timestamp, new_block_timestamp);

    state.stop_warp(contract_address);

    let output =
        call_contract(&contract_address, &selector, vec![].as_slice(), &mut state).unwrap();
    let changed_back_block_timestamp = recover_data(output);

    assert_eq!(old_block_timestamp, changed_back_block_timestamp);
}

#[test]
fn warp_double() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "WarpChecker", vec![].as_slice());

    let selector = felt_selector_from_name("get_block_timestamp");

    let output =
        call_contract(&contract_address, &selector, vec![].as_slice(), &mut state).unwrap();

    let old_block_timestamp = recover_data(output);

    state.start_warp(contract_address, Felt252::from(123_u128));
    state.start_warp(contract_address, Felt252::from(123_u128));

    let output =
        call_contract(&contract_address, &selector, vec![].as_slice(), &mut state).unwrap();

    let new_block_timestamp = recover_data(output);
    assert_eq!(new_block_timestamp, vec![Felt252::from(123)]);
    assert_ne!(old_block_timestamp, new_block_timestamp);

    state.stop_warp(contract_address);

    let output =
        call_contract(&contract_address, &selector, vec![].as_slice(), &mut state).unwrap();
    let changed_back_block_timestamp = recover_data(output);

    assert_eq!(old_block_timestamp, changed_back_block_timestamp);
}

#[test]
fn warp_proxy() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "WarpChecker", vec![].as_slice());

    state.start_warp(contract_address, Felt252::from(123_u128));

    let selector = felt_selector_from_name("get_block_timestamp");

    let output =
        call_contract(&contract_address, &selector, vec![].as_slice(), &mut state).unwrap();

    assert_success!(output, vec![Felt252::from(123)]);

    let proxy_address = deploy_contract(&mut state, "WarpCheckerProxy", vec![].as_slice());
    let proxy_selector = felt_selector_from_name("get_warp_checkers_block_timestamp");
    let output = call_contract(
        &proxy_address,
        &proxy_selector,
        vec![contract_address_to_felt(contract_address)].as_slice(),
        &mut state,
    )
    .unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn warp_library_call() {
    let mut state = create_cheatnet_state();

    let contracts = get_contracts();
    let contract_name = felt_from_short_string("WarpChecker");
    let class_hash = state.declare(&contract_name, &contracts).unwrap();

    let lib_call_address = deploy_contract(&mut state, "WarpCheckerLibCall", vec![].as_slice());

    state.start_warp(lib_call_address, Felt252::from(123_u128));

    let lib_call_selector = felt_selector_from_name("get_block_timestamp_with_lib_call");
    let output = call_contract(
        &lib_call_address,
        &lib_call_selector,
        vec![class_hash_to_felt(class_hash)].as_slice(),
        &mut state,
    )
    .unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}
