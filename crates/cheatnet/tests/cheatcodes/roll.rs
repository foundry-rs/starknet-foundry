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

#[test]
fn roll_simple() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "RollChecker",
        &[],
    );

    cheatnet_state.start_roll(contract_address, Felt252::from(123_u128));

    let selector = felt_selector_from_name("get_block_number");

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
fn roll_with_other_syscall() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "RollChecker",
        &[],
    );

    cheatnet_state.start_roll(contract_address, Felt252::from(123_u128));

    let selector = felt_selector_from_name("get_block_number_and_emit_event");

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
fn roll_in_constructor() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contracts = get_contracts();

    let contract_name = "ConstructorRollChecker".to_owned().to_felt252();
    let class_hash = blockifier_state
        .declare(&contract_name, &contracts)
        .unwrap();
    let precalculated_address = cheatnet_state.precalculate_address(&class_hash, &[]);

    cheatnet_state.start_roll(precalculated_address, Felt252::from(123_u128));

    let contract_address = deploy(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[])
        .unwrap()
        .contract_address;

    assert_eq!(precalculated_address, contract_address);

    let selector = felt_selector_from_name("get_stored_block_number");

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
fn roll_stop() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "RollChecker",
        &[],
    );

    let selector = felt_selector_from_name("get_block_number");

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

    let old_block_number = recover_data(output);

    cheatnet_state.start_roll(contract_address, Felt252::from(123_u128));

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

    let new_block_number = recover_data(output);
    assert_eq!(new_block_number, vec![Felt252::from(123)]);
    assert_ne!(old_block_number, new_block_number);

    cheatnet_state.stop_roll(contract_address);

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();
    let changed_back_block_number = recover_data(output);

    assert_eq!(old_block_number, changed_back_block_number);
}

#[test]
fn roll_double() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "RollChecker",
        &[],
    );

    let selector = felt_selector_from_name("get_block_number");

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

    let old_block_number = recover_data(output);

    cheatnet_state.start_roll(contract_address, Felt252::from(123_u128));
    cheatnet_state.start_roll(contract_address, Felt252::from(123_u128));

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

    let new_block_number = recover_data(output);
    assert_eq!(new_block_number, vec![Felt252::from(123)]);
    assert_ne!(old_block_number, new_block_number);

    cheatnet_state.stop_roll(contract_address);

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();
    let changed_back_block_number = recover_data(output);

    assert_eq!(old_block_number, changed_back_block_number);
}

#[test]
fn roll_proxy() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "RollChecker",
        &[],
    );

    let proxy_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "RollCheckerProxy",
        &[],
    );

    let proxy_selector = felt_selector_from_name("get_roll_checkers_block_number");
    let before_roll_output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &proxy_address,
        &proxy_selector,
        &[contract_address.to_felt252()],
    )
    .unwrap();

    cheatnet_state.start_roll(contract_address, Felt252::from(123_u128));

    let after_roll_output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &proxy_address,
        &proxy_selector,
        &[contract_address.to_felt252()],
    )
    .unwrap();

    assert_success!(after_roll_output, vec![Felt252::from(123)]);

    cheatnet_state.stop_roll(contract_address);

    let after_roll_cancellation_output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &proxy_address,
        &proxy_selector,
        &[contract_address.to_felt252()],
    )
    .unwrap();

    assert_outputs(before_roll_output, after_roll_cancellation_output);
}

#[test]
fn roll_library_call() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contracts = get_contracts();
    let contract_name = "RollChecker".to_owned().to_felt252();
    let class_hash = blockifier_state
        .declare(&contract_name, &contracts)
        .unwrap();

    let lib_call_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "RollCheckerLibCall",
        &[],
    );

    let lib_call_selector = felt_selector_from_name("get_block_number_with_lib_call");
    let before_roll_output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &lib_call_address,
        &lib_call_selector,
        &[class_hash.to_felt252()],
    )
    .unwrap();

    cheatnet_state.start_roll(lib_call_address, Felt252::from(123_u128));

    let after_roll_output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &lib_call_address,
        &lib_call_selector,
        &[class_hash.to_felt252()],
    )
    .unwrap();

    assert_success!(after_roll_output, vec![Felt252::from(123)]);

    cheatnet_state.stop_roll(lib_call_address);

    let after_roll_cancellation_output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &lib_call_address,
        &lib_call_selector,
        &[class_hash.to_felt252()],
    )
    .unwrap();

    assert_outputs(before_roll_output, after_roll_cancellation_output);
}
