use crate::common::assertions::assert_outputs;
use crate::common::state::build_runtime_state;
use crate::common::{call_contract, deploy_wrapper};
use crate::{
    assert_success,
    common::{
        deploy_contract, felt_selector_from_name, get_contracts, recover_data,
        state::{create_cached_state, create_runtime_states},
    },
};
use cairo_felt::Felt252;
use cheatnet::state::CheatTarget;
use conversions::felt252::FromShortString;
use conversions::IntoConv;
use starknet_api::core::ContractAddress;

#[test]
fn elect_simple() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut runtime_state_raw) = create_runtime_states(&mut cached_state);
    let mut runtime_state = build_runtime_state(&mut runtime_state_raw);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut runtime_state,
        "ElectChecker",
        &[],
    );

    runtime_state.cheatnet_state.start_elect(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );

    let selector = felt_selector_from_name("get_sequencer_address");

    let output = call_contract(
        &mut blockifier_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn elect_with_other_syscall() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut runtime_state_raw) = create_runtime_states(&mut cached_state);
    let mut runtime_state = build_runtime_state(&mut runtime_state_raw);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut runtime_state,
        "ElectChecker",
        &[],
    );

    runtime_state.cheatnet_state.start_elect(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );

    let selector = felt_selector_from_name("get_seq_addr_and_emit_event");

    let output = call_contract(
        &mut blockifier_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );
    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn elect_in_constructor() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut runtime_state_raw) = create_runtime_states(&mut cached_state);
    let mut runtime_state = build_runtime_state(&mut runtime_state_raw);

    let contracts = get_contracts();

    let contract_name = Felt252::from_short_string("ConstructorElectChecker").unwrap();
    let class_hash = blockifier_state
        .declare(&contract_name, &contracts)
        .unwrap();
    let precalculated_address = runtime_state
        .cheatnet_state
        .precalculate_address(&class_hash, &[]);

    runtime_state.cheatnet_state.start_elect(
        CheatTarget::One(precalculated_address),
        ContractAddress::from(123_u128),
    );

    let contract_address =
        deploy_wrapper(&mut blockifier_state, &mut runtime_state, &class_hash, &[]).unwrap();

    assert_eq!(precalculated_address, contract_address);

    let selector = felt_selector_from_name("get_stored_sequencer_address");

    let output = call_contract(
        &mut blockifier_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn elect_stop() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut runtime_state_raw) = create_runtime_states(&mut cached_state);
    let mut runtime_state = build_runtime_state(&mut runtime_state_raw);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut runtime_state,
        "ElectChecker",
        &[],
    );

    let selector = felt_selector_from_name("get_sequencer_address");

    let output = call_contract(
        &mut blockifier_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let old_sequencer_address = recover_data(output);

    runtime_state.cheatnet_state.start_elect(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );

    let output = call_contract(
        &mut blockifier_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let new_sequencer_address = recover_data(output);
    assert_eq!(new_sequencer_address, vec![Felt252::from(123)]);
    assert_ne!(old_sequencer_address, new_sequencer_address);

    runtime_state
        .cheatnet_state
        .stop_elect(CheatTarget::One(contract_address));

    let output = call_contract(
        &mut blockifier_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );
    let changed_back_sequencer_address = recover_data(output);

    assert_eq!(old_sequencer_address, changed_back_sequencer_address);
}

#[test]
fn elect_double() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut runtime_state_raw) = create_runtime_states(&mut cached_state);
    let mut runtime_state = build_runtime_state(&mut runtime_state_raw);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut runtime_state,
        "ElectChecker",
        &[],
    );

    let selector = felt_selector_from_name("get_sequencer_address");

    let output = call_contract(
        &mut blockifier_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let old_sequencer_address = recover_data(output);

    runtime_state.cheatnet_state.start_elect(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );
    runtime_state.cheatnet_state.start_elect(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );

    let output = call_contract(
        &mut blockifier_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let new_sequencer_address = recover_data(output);
    assert_eq!(new_sequencer_address, vec![Felt252::from(123)]);
    assert_ne!(old_sequencer_address, new_sequencer_address);

    runtime_state
        .cheatnet_state
        .stop_elect(CheatTarget::One(contract_address));

    let output = call_contract(
        &mut blockifier_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );
    let changed_back_sequencer_address = recover_data(output);

    assert_eq!(old_sequencer_address, changed_back_sequencer_address);
}

#[test]
fn elect_proxy() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut runtime_state_raw) = create_runtime_states(&mut cached_state);
    let mut runtime_state = build_runtime_state(&mut runtime_state_raw);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut runtime_state,
        "ElectChecker",
        &[],
    );

    let proxy_address = deploy_contract(
        &mut blockifier_state,
        &mut runtime_state,
        "ElectCheckerProxy",
        &[],
    );

    let proxy_selector = felt_selector_from_name("get_elect_checkers_seq_addr");
    let before_elect_output = call_contract(
        &mut blockifier_state,
        &mut runtime_state,
        &proxy_address,
        &proxy_selector,
        &[contract_address.into_()],
    );

    runtime_state.cheatnet_state.start_elect(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );

    let after_elect_output = call_contract(
        &mut blockifier_state,
        &mut runtime_state,
        &proxy_address,
        &proxy_selector,
        &[contract_address.into_()],
    );

    assert_success!(after_elect_output, vec![Felt252::from(123)]);

    runtime_state
        .cheatnet_state
        .stop_elect(CheatTarget::One(contract_address));

    let after_elect_cancellation_output = call_contract(
        &mut blockifier_state,
        &mut runtime_state,
        &proxy_address,
        &proxy_selector,
        &[contract_address.into_()],
    );

    assert_outputs(before_elect_output, after_elect_cancellation_output);
}

#[test]
fn elect_library_call() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut runtime_state_raw) = create_runtime_states(&mut cached_state);
    let mut runtime_state = build_runtime_state(&mut runtime_state_raw);

    let contracts = get_contracts();
    let contract_name = Felt252::from_short_string("ElectChecker").unwrap();
    let class_hash = blockifier_state
        .declare(&contract_name, &contracts)
        .unwrap();

    let lib_call_address = deploy_contract(
        &mut blockifier_state,
        &mut runtime_state,
        "ElectCheckerLibCall",
        &[],
    );

    let lib_call_selector = felt_selector_from_name("get_sequencer_address_with_lib_call");
    let before_elect_output = call_contract(
        &mut blockifier_state,
        &mut runtime_state,
        &lib_call_address,
        &lib_call_selector,
        &[class_hash.into_()],
    );

    runtime_state.cheatnet_state.start_elect(
        CheatTarget::One(lib_call_address),
        ContractAddress::from(123_u128),
    );

    let after_elect_output = call_contract(
        &mut blockifier_state,
        &mut runtime_state,
        &lib_call_address,
        &lib_call_selector,
        &[class_hash.into_()],
    );

    assert_success!(after_elect_output, vec![Felt252::from(123)]);

    runtime_state
        .cheatnet_state
        .stop_elect(CheatTarget::One(lib_call_address));

    let after_elect_cancellation_output = call_contract(
        &mut blockifier_state,
        &mut runtime_state,
        &lib_call_address,
        &lib_call_selector,
        &[class_hash.into_()],
    );

    assert_outputs(before_elect_output, after_elect_cancellation_output);
}

#[test]
fn elect_all_simple() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut runtime_state_raw) = create_runtime_states(&mut cached_state);
    let mut runtime_state = build_runtime_state(&mut runtime_state_raw);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut runtime_state,
        "ElectChecker",
        &[],
    );

    runtime_state
        .cheatnet_state
        .start_elect(CheatTarget::All, ContractAddress::from(123_u128));

    let selector = felt_selector_from_name("get_sequencer_address");

    let output = call_contract(
        &mut blockifier_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn elect_all_then_one() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut runtime_state_raw) = create_runtime_states(&mut cached_state);
    let mut runtime_state = build_runtime_state(&mut runtime_state_raw);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut runtime_state,
        "ElectChecker",
        &[],
    );

    runtime_state
        .cheatnet_state
        .start_elect(CheatTarget::All, ContractAddress::from(321_u128));
    runtime_state.cheatnet_state.start_elect(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );

    let selector = felt_selector_from_name("get_sequencer_address");

    let output = call_contract(
        &mut blockifier_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn elect_one_then_all() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut runtime_state_raw) = create_runtime_states(&mut cached_state);
    let mut runtime_state = build_runtime_state(&mut runtime_state_raw);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut runtime_state,
        "ElectChecker",
        &[],
    );

    runtime_state.cheatnet_state.start_elect(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );
    runtime_state
        .cheatnet_state
        .start_elect(CheatTarget::All, ContractAddress::from(321_u128));

    let selector = felt_selector_from_name("get_sequencer_address");

    let output = call_contract(
        &mut blockifier_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success!(output, vec![Felt252::from(321)]);
}

#[test]
fn elect_all_stop() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut runtime_state_raw) = create_runtime_states(&mut cached_state);
    let mut runtime_state = build_runtime_state(&mut runtime_state_raw);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut runtime_state,
        "ElectChecker",
        &[],
    );

    let selector = felt_selector_from_name("get_sequencer_address");

    let output = call_contract(
        &mut blockifier_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let old_sequencer_address = recover_data(output);

    runtime_state
        .cheatnet_state
        .start_elect(CheatTarget::All, ContractAddress::from(123_u128));

    let output = call_contract(
        &mut blockifier_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let new_sequencer_address = recover_data(output);
    assert_eq!(new_sequencer_address, vec![Felt252::from(123)]);
    assert_ne!(old_sequencer_address, new_sequencer_address);

    runtime_state.cheatnet_state.stop_elect(CheatTarget::All);

    let output = call_contract(
        &mut blockifier_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );
    let changed_back_sequencer_address = recover_data(output);

    assert_eq!(old_sequencer_address, changed_back_sequencer_address);
}

#[test]
fn elect_multiple() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut runtime_state_raw) = create_runtime_states(&mut cached_state);
    let mut runtime_state = build_runtime_state(&mut runtime_state_raw);

    let contract = Felt252::from_short_string("ElectChecker").unwrap();
    let contracts = get_contracts();
    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();

    let contract_address1 =
        deploy_wrapper(&mut blockifier_state, &mut runtime_state, &class_hash, &[]).unwrap();

    let contract_address2 =
        deploy_wrapper(&mut blockifier_state, &mut runtime_state, &class_hash, &[]).unwrap();

    let selector = felt_selector_from_name("get_sequencer_address");

    let output = call_contract(
        &mut blockifier_state,
        &mut runtime_state,
        &contract_address1,
        &selector,
        &[],
    );

    let old_sequencer_address1 = recover_data(output);

    let output = call_contract(
        &mut blockifier_state,
        &mut runtime_state,
        &contract_address2,
        &selector,
        &[],
    );

    let old_sequencer_address2 = recover_data(output);

    runtime_state.cheatnet_state.start_elect(
        CheatTarget::Multiple(vec![contract_address1, contract_address2]),
        ContractAddress::from(123_u128),
    );

    let output = call_contract(
        &mut blockifier_state,
        &mut runtime_state,
        &contract_address1,
        &selector,
        &[],
    );

    let new_sequencer_address1 = recover_data(output);

    let output = call_contract(
        &mut blockifier_state,
        &mut runtime_state,
        &contract_address2,
        &selector,
        &[],
    );

    let new_sequencer_address2 = recover_data(output);

    assert_eq!(new_sequencer_address1, vec![Felt252::from(123)]);
    assert_eq!(new_sequencer_address2, vec![Felt252::from(123)]);

    runtime_state
        .cheatnet_state
        .stop_elect(CheatTarget::Multiple(vec![
            contract_address1,
            contract_address2,
        ]));

    let output = call_contract(
        &mut blockifier_state,
        &mut runtime_state,
        &contract_address1,
        &selector,
        &[],
    );

    let changed_back_sequencer_address1 = recover_data(output);

    let output = call_contract(
        &mut blockifier_state,
        &mut runtime_state,
        &contract_address2,
        &selector,
        &[],
    );

    let changed_back_sequencer_address2 = recover_data(output);

    assert_eq!(old_sequencer_address1, changed_back_sequencer_address1);
    assert_eq!(old_sequencer_address2, changed_back_sequencer_address2);
}
