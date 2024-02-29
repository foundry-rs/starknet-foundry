use crate::common::assertions::assert_outputs;
use crate::common::state::build_runtime_state;
use crate::common::{call_contract, deploy_wrapper};
use crate::{
    assert_success,
    common::{deploy_contract, felt_selector_from_name, get_contracts, state::create_cached_state},
};
use cairo_felt::Felt252;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::declare;
use cheatnet::state::{CheatTarget, CheatnetState};
use conversions::IntoConv;
use starknet_api::core::ContractAddress;

use super::test_environment::TestEnvironment;

trait ElectTrait {
    fn start_elect(&mut self, target: CheatTarget, sequencer_address: u128);
    fn stop_elect(&mut self, contract_address: &ContractAddress);
}

impl<'a> ElectTrait for TestEnvironment<'a> {
    fn start_elect(&mut self, target: CheatTarget, sequencer_address: u128) {
        self.runtime_state
            .cheatnet_state
            .start_elect(target, ContractAddress::from(sequencer_address));
    }

    fn stop_elect(&mut self, contract_address: &ContractAddress) {
        self.runtime_state
            .cheatnet_state
            .stop_elect(CheatTarget::One(*contract_address));
    }
}

#[test]
fn elect_simple() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "ElectChecker", &[]);

    runtime_state.cheatnet_state.start_elect(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );

    let selector = felt_selector_from_name("get_sequencer_address");

    let output = call_contract(
        &mut cached_state,
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
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "ElectChecker", &[]);

    runtime_state.cheatnet_state.start_elect(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );

    let selector = felt_selector_from_name("get_seq_addr_and_emit_event");

    let output = call_contract(
        &mut cached_state,
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
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contracts = get_contracts();

    let class_hash = declare(&mut cached_state, "ConstructorElectChecker", &contracts).unwrap();
    let precalculated_address = runtime_state
        .cheatnet_state
        .precalculate_address(&class_hash, &[]);

    runtime_state.cheatnet_state.start_elect(
        CheatTarget::One(precalculated_address),
        ContractAddress::from(123_u128),
    );

    let contract_address =
        deploy_wrapper(&mut cached_state, &mut runtime_state, &class_hash, &[]).unwrap();

    assert_eq!(precalculated_address, contract_address);

    let selector = felt_selector_from_name("get_stored_sequencer_address");

    let output = call_contract(
        &mut cached_state,
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
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "ElectChecker", &[]);

    let selector = felt_selector_from_name("get_sequencer_address");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let old_sequencer_address = output.recover_data();

    runtime_state.cheatnet_state.start_elect(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let new_sequencer_address = output.recover_data();
    assert_eq!(new_sequencer_address, vec![Felt252::from(123)]);
    assert_ne!(old_sequencer_address, new_sequencer_address);

    runtime_state
        .cheatnet_state
        .stop_elect(CheatTarget::One(contract_address));

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );
    let changed_back_sequencer_address = output.recover_data();

    assert_eq!(old_sequencer_address, changed_back_sequencer_address);
}

#[test]
fn elect_double() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "ElectChecker", &[]);

    let selector = felt_selector_from_name("get_sequencer_address");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let old_sequencer_address = output.recover_data();

    runtime_state.cheatnet_state.start_elect(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );
    runtime_state.cheatnet_state.start_elect(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let new_sequencer_address = output.recover_data();
    assert_eq!(new_sequencer_address, vec![Felt252::from(123)]);
    assert_ne!(old_sequencer_address, new_sequencer_address);

    runtime_state
        .cheatnet_state
        .stop_elect(CheatTarget::One(contract_address));

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );
    let changed_back_sequencer_address = output.recover_data();

    assert_eq!(old_sequencer_address, changed_back_sequencer_address);
}

#[test]
fn elect_proxy() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "ElectChecker", &[]);

    let proxy_address = deploy_contract(
        &mut cached_state,
        &mut runtime_state,
        "ElectCheckerProxy",
        &[],
    );

    let proxy_selector = felt_selector_from_name("get_elect_checkers_seq_addr");
    let before_elect_output = call_contract(
        &mut cached_state,
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
        &mut cached_state,
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
        &mut cached_state,
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
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contracts = get_contracts();
    let class_hash = declare(&mut cached_state, "ElectChecker", &contracts).unwrap();

    let lib_call_address = deploy_contract(
        &mut cached_state,
        &mut runtime_state,
        "ElectCheckerLibCall",
        &[],
    );

    let lib_call_selector = felt_selector_from_name("get_sequencer_address_with_lib_call");
    let before_elect_output = call_contract(
        &mut cached_state,
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
        &mut cached_state,
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
        &mut cached_state,
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
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "ElectChecker", &[]);

    runtime_state
        .cheatnet_state
        .start_elect(CheatTarget::All, ContractAddress::from(123_u128));

    let selector = felt_selector_from_name("get_sequencer_address");

    let output = call_contract(
        &mut cached_state,
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
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "ElectChecker", &[]);

    runtime_state
        .cheatnet_state
        .start_elect(CheatTarget::All, ContractAddress::from(321_u128));
    runtime_state.cheatnet_state.start_elect(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );

    let selector = felt_selector_from_name("get_sequencer_address");

    let output = call_contract(
        &mut cached_state,
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
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "ElectChecker", &[]);

    runtime_state.cheatnet_state.start_elect(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );
    runtime_state
        .cheatnet_state
        .start_elect(CheatTarget::All, ContractAddress::from(321_u128));

    let selector = felt_selector_from_name("get_sequencer_address");

    let output = call_contract(
        &mut cached_state,
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
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "ElectChecker", &[]);

    let selector = felt_selector_from_name("get_sequencer_address");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let old_sequencer_address = output.recover_data();

    runtime_state
        .cheatnet_state
        .start_elect(CheatTarget::All, ContractAddress::from(123_u128));

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let new_sequencer_address = output.recover_data();
    assert_eq!(new_sequencer_address, vec![Felt252::from(123)]);
    assert_ne!(old_sequencer_address, new_sequencer_address);

    runtime_state.cheatnet_state.stop_elect(CheatTarget::All);

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );
    let changed_back_sequencer_address = output.recover_data();

    assert_eq!(old_sequencer_address, changed_back_sequencer_address);
}

#[test]
fn elect_multiple() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contracts = get_contracts();
    let class_hash = declare(&mut cached_state, "ElectChecker", &contracts).unwrap();

    let contract_address1 =
        deploy_wrapper(&mut cached_state, &mut runtime_state, &class_hash, &[]).unwrap();

    let contract_address2 =
        deploy_wrapper(&mut cached_state, &mut runtime_state, &class_hash, &[]).unwrap();

    let selector = felt_selector_from_name("get_sequencer_address");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address1,
        &selector,
        &[],
    );

    let old_sequencer_address1 = output.recover_data();

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address2,
        &selector,
        &[],
    );

    let old_sequencer_address2 = output.recover_data();

    runtime_state.cheatnet_state.start_elect(
        CheatTarget::Multiple(vec![contract_address1, contract_address2]),
        ContractAddress::from(123_u128),
    );

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address1,
        &selector,
        &[],
    );

    let new_sequencer_address1 = output.recover_data();

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address2,
        &selector,
        &[],
    );

    let new_sequencer_address2 = output.recover_data();

    assert_eq!(new_sequencer_address1, vec![Felt252::from(123)]);
    assert_eq!(new_sequencer_address2, vec![Felt252::from(123)]);

    runtime_state
        .cheatnet_state
        .stop_elect(CheatTarget::Multiple(vec![
            contract_address1,
            contract_address2,
        ]));

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address1,
        &selector,
        &[],
    );

    let changed_back_sequencer_address1 = output.recover_data();

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address2,
        &selector,
        &[],
    );

    let changed_back_sequencer_address2 = output.recover_data();

    assert_eq!(old_sequencer_address1, changed_back_sequencer_address1);
    assert_eq!(old_sequencer_address2, changed_back_sequencer_address2);
}
