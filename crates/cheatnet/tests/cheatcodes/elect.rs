use super::test_environment::TestEnvironment;
use crate::common::assertions::assert_outputs;
use crate::common::state::build_runtime_state;
use crate::common::{call_contract, deploy_wrapper};
use crate::{
    common::assertions::assert_success,
    common::{
        deploy_contract, felt_selector_from_name, get_contracts, recover_data,
        state::create_cached_state,
    },
};
use cairo_felt::Felt252;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::declare;
use cheatnet::state::{CheatSpan, CheatTarget, CheatnetState};
use conversions::IntoConv;
use runtime::starknet::context::SEQUENCER_ADDRESS;
use starknet_api::core::{ContractAddress, PatriciaKey};
use starknet_api::hash::StarkHash;
use starknet_api::{contract_address, patricia_key};

trait ElectTrait {
    fn elect(&mut self, target: CheatTarget, sequencer_address: u128, span: CheatSpan);
    fn start_elect(&mut self, target: CheatTarget, sequencer_address: u128);
    fn stop_elect(&mut self, contract_address: &ContractAddress);
}

impl<'a> ElectTrait for TestEnvironment<'a> {
    fn elect(&mut self, target: CheatTarget, sequencer_address: u128, span: CheatSpan) {
        self.runtime_state.cheatnet_state.elect(
            target,
            ContractAddress::from(sequencer_address),
            span,
        );
    }

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

    assert_success(output, &[Felt252::from(123)]);
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
    assert_success(output, &[Felt252::from(123)]);
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

    assert_success(output, &[Felt252::from(123)]);
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

    let old_sequencer_address = recover_data(output);

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

    let new_sequencer_address = recover_data(output);
    assert_eq!(new_sequencer_address, &[Felt252::from(123)]);
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
    let changed_back_sequencer_address = recover_data(output);

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
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let new_sequencer_address = recover_data(output);
    assert_eq!(new_sequencer_address, &[Felt252::from(123)]);
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
    let changed_back_sequencer_address = recover_data(output);

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

    assert_success(after_elect_output, &[Felt252::from(123)]);

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

    assert_success(after_elect_output, &[Felt252::from(123)]);

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

    assert_success(output, &[Felt252::from(123)]);
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

    assert_success(output, &[Felt252::from(123)]);
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

    assert_success(output, &[Felt252::from(321)]);
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

    let old_sequencer_address = recover_data(output);

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

    let new_sequencer_address = recover_data(output);
    assert_eq!(new_sequencer_address, &[Felt252::from(123)]);
    assert_ne!(old_sequencer_address, new_sequencer_address);

    runtime_state.cheatnet_state.stop_elect(CheatTarget::All);

    let output = call_contract(
        &mut cached_state,
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

    let old_sequencer_address1 = recover_data(output);

    let output = call_contract(
        &mut cached_state,
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
        &mut cached_state,
        &mut runtime_state,
        &contract_address1,
        &selector,
        &[],
    );

    let new_sequencer_address1 = recover_data(output);

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address2,
        &selector,
        &[],
    );

    let new_sequencer_address2 = recover_data(output);

    assert_eq!(new_sequencer_address1, &[Felt252::from(123)]);
    assert_eq!(new_sequencer_address2, &[Felt252::from(123)]);

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

    let changed_back_sequencer_address1 = recover_data(output);

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address2,
        &selector,
        &[],
    );

    let changed_back_sequencer_address2 = recover_data(output);

    assert_eq!(old_sequencer_address1, changed_back_sequencer_address1);
    assert_eq!(old_sequencer_address2, changed_back_sequencer_address2);
}

#[test]
fn elect_simple_with_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address = test_env.deploy("ElectChecker", &[]);

    test_env.elect(
        CheatTarget::One(contract_address),
        123,
        CheatSpan::Number(2),
    );

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[contract_address!(SEQUENCER_ADDRESS).into_()],
    );
}

#[test]
fn elect_proxy_with_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contracts = get_contracts();
    let class_hash = test_env.declare("ElectCheckerProxy", &contracts);
    let contract_address_1 = test_env.deploy_wrapper(&class_hash, &[]);
    let contract_address_2 = test_env.deploy_wrapper(&class_hash, &[]);

    test_env.elect(
        CheatTarget::One(contract_address_1),
        123,
        CheatSpan::Number(1),
    );

    let output = test_env.call_contract(
        &contract_address_1,
        "call_proxy",
        &[contract_address_2.into_()],
    );
    assert_success(
        output,
        &[123.into(), contract_address!(SEQUENCER_ADDRESS).into_()],
    );
}

#[test]
fn elect_in_constructor_with_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contracts = get_contracts();

    let class_hash = test_env.declare("ConstructorElectChecker", &contracts);
    let precalculated_address = test_env
        .runtime_state
        .cheatnet_state
        .precalculate_address(&class_hash, &[]);

    test_env.elect(
        CheatTarget::One(precalculated_address),
        123,
        CheatSpan::Number(2),
    );

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);
    assert_eq!(precalculated_address, contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[contract_address!(SEQUENCER_ADDRESS).into_()],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_stored_sequencer_address", &[]),
        &[Felt252::from(123)],
    );
}

#[test]
fn elect_no_constructor_with_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contracts = get_contracts();

    let class_hash = test_env.declare("ElectChecker", &contracts);
    let precalculated_address = test_env
        .runtime_state
        .cheatnet_state
        .precalculate_address(&class_hash, &[]);

    test_env.elect(
        CheatTarget::One(precalculated_address),
        123,
        CheatSpan::Number(1),
    );

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);
    assert_eq!(precalculated_address, contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[contract_address!(SEQUENCER_ADDRESS).into_()],
    );
}

#[test]
fn elect_override_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address = test_env.deploy("ElectChecker", &[]);

    test_env.elect(
        CheatTarget::One(contract_address),
        123,
        CheatSpan::Number(2),
    );

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[Felt252::from(123)],
    );

    test_env.elect(
        CheatTarget::One(contract_address),
        321,
        CheatSpan::Indefinite,
    );

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[Felt252::from(321)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[Felt252::from(321)],
    );

    test_env.stop_elect(&contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[contract_address!(SEQUENCER_ADDRESS).into_()],
    );
}

#[test]
fn elect_library_call_with_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contracts = get_contracts();
    let class_hash = test_env.declare("ElectChecker", &contracts);
    let contract_address = test_env.deploy("ElectCheckerLibCall", &[]);

    test_env.elect(
        CheatTarget::One(contract_address),
        123,
        CheatSpan::Number(1),
    );

    let lib_call_selector = "get_sequencer_address_with_lib_call";

    assert_success(
        test_env.call_contract(&contract_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, lib_call_selector, &[class_hash.into_()]),
        &[contract_address!(SEQUENCER_ADDRESS).into_()],
    );
}

#[test]
fn elect_all_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address_1 = test_env.deploy("ElectChecker", &[]);
    let contract_address_2 = test_env.deploy("ElectCheckerLibCall", &[]);

    test_env.elect(CheatTarget::All, 123, CheatSpan::Number(1));

    assert_success(
        test_env.call_contract(&contract_address_1, "get_sequencer_address", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address_1, "get_sequencer_address", &[]),
        &[contract_address!(SEQUENCER_ADDRESS).into_()],
    );

    assert_success(
        test_env.call_contract(&contract_address_2, "get_sequencer_address", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address_2, "get_sequencer_address", &[]),
        &[contract_address!(SEQUENCER_ADDRESS).into_()],
    );
}
