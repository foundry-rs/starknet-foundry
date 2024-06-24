use super::test_environment::TestEnvironment;
use crate::{common::assertions::assert_success, common::get_contracts};
use cairo_felt::Felt252;
use cheatnet::state::{CheatSpan, CheatTarget};
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

impl ElectTrait for TestEnvironment {
    fn elect(&mut self, target: CheatTarget, sequencer_address: u128, span: CheatSpan) {
        self.cheatnet_state
            .elect(target, ContractAddress::from(sequencer_address), span);
    }

    fn start_elect(&mut self, target: CheatTarget, sequencer_address: u128) {
        self.cheatnet_state
            .start_elect(target, ContractAddress::from(sequencer_address));
    }

    fn stop_elect(&mut self, contract_address: &ContractAddress) {
        self.cheatnet_state
            .stop_elect(CheatTarget::One(*contract_address));
    }
}

#[test]
fn elect_simple() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("ElectChecker", &[]);

    test_env.start_elect(CheatTarget::One(contract_address), 123);

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[Felt252::from(123)],
    );
}

#[test]
fn elect_with_other_syscall() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("ElectChecker", &[]);

    test_env.start_elect(CheatTarget::One(contract_address), 123);

    assert_success(
        test_env.call_contract(&contract_address, "get_seq_addr_and_emit_event", &[]),
        &[Felt252::from(123)],
    );
}

#[test]
fn elect_in_constructor() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();

    let class_hash = test_env.declare("ConstructorElectChecker", &contracts_data);
    let precalculated_address = test_env.precalculate_address(&class_hash, &[]);

    test_env.start_elect(CheatTarget::One(precalculated_address), 123);

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);
    assert_eq!(precalculated_address, contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_stored_sequencer_address", &[]),
        &[Felt252::from(123)],
    );
}

#[test]
fn elect_stop() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("ElectChecker", &[]);

    test_env.start_elect(CheatTarget::One(contract_address), 123);

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[Felt252::from(123)],
    );

    test_env
        .cheatnet_state
        .stop_elect(CheatTarget::One(contract_address));

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[contract_address!(SEQUENCER_ADDRESS).into_()],
    );
}

#[test]
fn elect_double() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("ElectChecker", &[]);

    test_env.start_elect(CheatTarget::One(contract_address), 111);
    test_env.start_elect(CheatTarget::One(contract_address), 222);

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[Felt252::from(222)],
    );

    test_env.stop_elect(&contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[contract_address!(SEQUENCER_ADDRESS).into_()],
    );
}

#[test]
fn elect_proxy() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("ElectChecker", &[]);
    let proxy_address = test_env.deploy("ElectCheckerProxy", &[]);

    test_env.start_elect(CheatTarget::One(contract_address), 123);

    let selector = "get_elect_checkers_seq_addr";
    assert_success(
        test_env.call_contract(&proxy_address, selector, &[contract_address.into_()]),
        &[Felt252::from(123)],
    );

    test_env.stop_elect(&contract_address);

    assert_success(
        test_env.call_contract(&proxy_address, selector, &[contract_address.into_()]),
        &[contract_address!(SEQUENCER_ADDRESS).into_()],
    );
}

#[test]
fn elect_library_call() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("ElectChecker", &contracts_data);

    let lib_call_address = test_env.deploy("ElectCheckerLibCall", &[]);
    let lib_call_selector = "get_sequencer_address_with_lib_call";

    test_env.start_elect(CheatTarget::One(lib_call_address), 123);

    assert_success(
        test_env.call_contract(&lib_call_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt252::from(123)],
    );

    test_env.stop_elect(&lib_call_address);

    assert_success(
        test_env.call_contract(&lib_call_address, lib_call_selector, &[class_hash.into_()]),
        &[contract_address!(SEQUENCER_ADDRESS).into_()],
    );
}

#[test]
fn elect_all_simple() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("ElectChecker", &[]);

    test_env.start_elect(CheatTarget::All, 123);

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[Felt252::from(123)],
    );
}

#[test]
fn elect_all_then_one() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("ElectChecker", &[]);

    test_env.start_elect(CheatTarget::All, 111);
    test_env.start_elect(CheatTarget::One(contract_address), 222);

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[Felt252::from(222)],
    );
}

#[test]
fn elect_one_then_all() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("ElectChecker", &[]);

    test_env.start_elect(CheatTarget::One(contract_address), 111);
    test_env.start_elect(CheatTarget::All, 222);

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[Felt252::from(222)],
    );
}

#[test]
fn elect_all_stop() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("ElectChecker", &[]);

    test_env.start_elect(CheatTarget::All, 123);

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[Felt252::from(123)],
    );

    test_env.cheatnet_state.stop_elect(CheatTarget::All);

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[contract_address!(SEQUENCER_ADDRESS).into_()],
    );
}

#[test]
fn elect_multiple() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("ElectChecker", &contracts_data);

    let contract_address1 = test_env.deploy_wrapper(&class_hash, &[]);
    let contract_address2 = test_env.deploy_wrapper(&class_hash, &[]);

    test_env.start_elect(
        CheatTarget::Multiple(vec![contract_address1, contract_address2]),
        123,
    );

    assert_success(
        test_env.call_contract(&contract_address1, "get_sequencer_address", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address2, "get_sequencer_address", &[]),
        &[Felt252::from(123)],
    );

    test_env
        .cheatnet_state
        .stop_elect(CheatTarget::Multiple(vec![
            contract_address1,
            contract_address2,
        ]));

    assert_success(
        test_env.call_contract(&contract_address1, "get_sequencer_address", &[]),
        &[contract_address!(SEQUENCER_ADDRESS).into_()],
    );
    assert_success(
        test_env.call_contract(&contract_address2, "get_sequencer_address", &[]),
        &[contract_address!(SEQUENCER_ADDRESS).into_()],
    );
}

#[test]
fn elect_simple_with_span() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("ElectChecker", &[]);

    test_env.elect(
        CheatTarget::One(contract_address),
        123,
        CheatSpan::TargetCalls(2),
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
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("ElectCheckerProxy", &contracts_data);
    let contract_address_1 = test_env.deploy_wrapper(&class_hash, &[]);
    let contract_address_2 = test_env.deploy_wrapper(&class_hash, &[]);

    test_env.elect(
        CheatTarget::One(contract_address_1),
        123,
        CheatSpan::TargetCalls(1),
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
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();

    let class_hash = test_env.declare("ConstructorElectChecker", &contracts_data);
    let precalculated_address = test_env
        .cheatnet_state
        .precalculate_address(&class_hash, &[]);

    test_env.elect(
        CheatTarget::One(precalculated_address),
        123,
        CheatSpan::TargetCalls(2),
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
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();

    let class_hash = test_env.declare("ElectChecker", &contracts_data);
    let precalculated_address = test_env
        .cheatnet_state
        .precalculate_address(&class_hash, &[]);

    test_env.elect(
        CheatTarget::One(precalculated_address),
        123,
        CheatSpan::TargetCalls(1),
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
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("ElectChecker", &[]);

    test_env.elect(
        CheatTarget::One(contract_address),
        123,
        CheatSpan::TargetCalls(2),
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
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("ElectChecker", &contracts_data);
    let contract_address = test_env.deploy("ElectCheckerLibCall", &[]);

    test_env.elect(
        CheatTarget::One(contract_address),
        123,
        CheatSpan::TargetCalls(1),
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
    let mut test_env = TestEnvironment::new();

    let contract_address_1 = test_env.deploy("ElectChecker", &[]);
    let contract_address_2 = test_env.deploy("ElectCheckerLibCall", &[]);

    test_env.elect(CheatTarget::All, 123, CheatSpan::TargetCalls(1));

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
