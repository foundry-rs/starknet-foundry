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
use conversions::IntoConv;
use starknet_api::core::ContractAddress;

#[test]
fn elect_simple() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "ElectChecker",
        &[],
    );

    cheatnet_state.start_elect(contract_address, ContractAddress::from(123_u128));

    let selector = felt_selector_from_name("get_sequencer_address");

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
fn elect_with_other_syscall() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "ElectChecker",
        &[],
    );

    cheatnet_state.start_elect(contract_address, ContractAddress::from(123_u128));

    let selector = felt_selector_from_name("get_seq_addr_and_emit_event");

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();
    dbg!(&output);
    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn elect_in_constructor() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contracts = get_contracts();

    let contract_name = "ConstructorElectChecker".to_owned().into_();
    let class_hash = blockifier_state
        .declare(&contract_name, &contracts)
        .unwrap();
    let precalculated_address = cheatnet_state.precalculate_address(&class_hash, &[]);

    cheatnet_state.start_elect(precalculated_address, ContractAddress::from(123_u128));

    let contract_address = deploy(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[])
        .unwrap()
        .contract_address;

    assert_eq!(precalculated_address, contract_address);

    let selector = felt_selector_from_name("get_stored_sequencer_address");

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
fn elect_stop() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "ElectChecker",
        &[],
    );

    let selector = felt_selector_from_name("get_sequencer_address");

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

    let old_sequencer_address = recover_data(output);

    cheatnet_state.start_elect(contract_address, ContractAddress::from(123_u128));

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

    let new_sequencer_address = recover_data(output);
    assert_eq!(new_sequencer_address, vec![Felt252::from(123)]);
    assert_ne!(old_sequencer_address, new_sequencer_address);

    cheatnet_state.stop_elect(contract_address);

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();
    let changed_back_sequencer_address = recover_data(output);

    assert_eq!(old_sequencer_address, changed_back_sequencer_address);
}

#[test]
fn elect_double() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "ElectChecker",
        &[],
    );

    let selector = felt_selector_from_name("get_sequencer_address");

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

    let old_sequencer_address = recover_data(output);

    cheatnet_state.start_elect(contract_address, ContractAddress::from(123_u128));
    cheatnet_state.start_elect(contract_address, ContractAddress::from(123_u128));

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

    let new_sequencer_address = recover_data(output);
    assert_eq!(new_sequencer_address, vec![Felt252::from(123)]);
    assert_ne!(old_sequencer_address, new_sequencer_address);

    cheatnet_state.stop_elect(contract_address);

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();
    let changed_back_sequencer_address = recover_data(output);

    assert_eq!(old_sequencer_address, changed_back_sequencer_address);
}

#[test]
fn elect_proxy() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "ElectChecker",
        &[],
    );

    let proxy_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "ElectCheckerProxy",
        &[],
    );

    let proxy_selector = felt_selector_from_name("get_elect_checkers_seq_addr");
    let before_elect_output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &proxy_address,
        &proxy_selector,
        &[contract_address.into_()],
    )
    .unwrap();

    cheatnet_state.start_elect(contract_address, ContractAddress::from(123_u128));

    let after_elect_output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &proxy_address,
        &proxy_selector,
        &[contract_address.into_()],
    )
    .unwrap();

    assert_success!(after_elect_output, vec![Felt252::from(123)]);

    cheatnet_state.stop_elect(contract_address);

    let after_elect_cancellation_output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &proxy_address,
        &proxy_selector,
        &[contract_address.into_()],
    )
    .unwrap();

    assert_outputs(before_elect_output, after_elect_cancellation_output);
}

#[test]
fn elect_library_call() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contracts = get_contracts();
    let contract_name = "ElectChecker".to_owned().into_();
    let class_hash = blockifier_state
        .declare(&contract_name, &contracts)
        .unwrap();

    let lib_call_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "ElectCheckerLibCall",
        &[],
    );

    let lib_call_selector = felt_selector_from_name("get_sequencer_address_with_lib_call");
    let before_elect_output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &lib_call_address,
        &lib_call_selector,
        &[class_hash.into_()],
    )
    .unwrap();

    cheatnet_state.start_elect(lib_call_address, ContractAddress::from(123_u128));

    let after_elect_output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &lib_call_address,
        &lib_call_selector,
        &[class_hash.into_()],
    )
    .unwrap();

    assert_success!(after_elect_output, vec![Felt252::from(123)]);

    cheatnet_state.stop_elect(lib_call_address);

    let after_elect_cancellation_output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &lib_call_address,
        &lib_call_selector,
        &[class_hash.into_()],
    )
    .unwrap();

    assert_outputs(before_elect_output, after_elect_cancellation_output);
}
