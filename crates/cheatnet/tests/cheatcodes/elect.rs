use super::test_environment::TestEnvironment;
use crate::{common::assertions::assert_success, common::get_contracts};
use cairo_felt::Felt252;
use cheatnet::state::CheatSpan;
use conversions::IntoConv;
use runtime::starknet::context::SEQUENCER_ADDRESS;
use starknet_api::core::{ContractAddress, PatriciaKey};
use starknet_api::hash::StarkHash;
use starknet_api::{contract_address, patricia_key};

trait CheatSequencerAddressTrait {
    fn cheat_sequencer_address(
        &mut self,
        contract_address: ContractAddress,
        sequencer_address: u128,
        span: CheatSpan,
    );
    fn start_cheat_sequencer_address(&mut self, contract_address: ContractAddress, sequencer_address: u128);
    fn stop_cheat_sequencer_address(&mut self, contract_address: ContractAddress);
}

impl CheatSequencerAddressTrait for TestEnvironment {
    fn cheat_sequencer_address(
        &mut self,
        contract_address: ContractAddress,
        sequencer_address: u128,
        span: CheatSpan,
    ) {
        self.cheatnet_state.cheat_sequencer_address(
            contract_address,
            ContractAddress::from(sequencer_address),
            span,
        );
    }

    fn start_cheat_sequencer_address(&mut self, contract_address: ContractAddress, sequencer_address: u128) {
        self.cheatnet_state
            .start_cheat_sequencer_address(contract_address, ContractAddress::from(sequencer_address));
    }

    fn stop_cheat_sequencer_address(&mut self, contract_address: ContractAddress) {
        self.cheatnet_state.stop_cheat_sequencer_address(contract_address);
    }
}

#[test]
fn cheat_sequencer_address_simple() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatSequencerAddressChecker", &[]);

    test_env.start_cheat_sequencer_address(contract_address, 123);

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[Felt252::from(123)],
    );
}

#[test]
fn cheat_sequencer_address_with_other_syscall() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatSequencerAddressChecker", &[]);

    test_env.start_cheat_sequencer_address(contract_address, 123);

    assert_success(
        test_env.call_contract(&contract_address, "get_seq_addr_and_emit_event", &[]),
        &[Felt252::from(123)],
    );
}

#[test]
fn cheat_sequencer_address_in_constructor() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();

    let class_hash = test_env.declare("ConstructorCheatSequencerAddressChecker", &contracts_data);
    let precalculated_address = test_env.precalculate_address(&class_hash, &[]);

    test_env.start_cheat_sequencer_address(precalculated_address, 123);

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);
    assert_eq!(precalculated_address, contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_stored_sequencer_address", &[]),
        &[Felt252::from(123)],
    );
}

#[test]
fn cheat_sequencer_address_stop() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatSequencerAddressChecker", &[]);

    test_env.start_cheat_sequencer_address(contract_address, 123);

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[Felt252::from(123)],
    );

    test_env.cheatnet_state.stop_cheat_sequencer_address(contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[contract_address!(SEQUENCER_ADDRESS).into_()],
    );
}

#[test]
fn cheat_sequencer_address_double() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatSequencerAddressChecker", &[]);

    test_env.start_cheat_sequencer_address(contract_address, 111);
    test_env.start_cheat_sequencer_address(contract_address, 222);

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[Felt252::from(222)],
    );

    test_env.stop_cheat_sequencer_address(contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[contract_address!(SEQUENCER_ADDRESS).into_()],
    );
}

#[test]
fn cheat_sequencer_address_proxy() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatSequencerAddressChecker", &[]);
    let proxy_address = test_env.deploy("CheatSequencerAddressCheckerProxy", &[]);

    test_env.start_cheat_sequencer_address(contract_address, 123);

    let selector = "get_cheat_sequencer_address_checkers_seq_addr";
    assert_success(
        test_env.call_contract(&proxy_address, selector, &[contract_address.into_()]),
        &[Felt252::from(123)],
    );

    test_env.stop_cheat_sequencer_address(contract_address);

    assert_success(
        test_env.call_contract(&proxy_address, selector, &[contract_address.into_()]),
        &[contract_address!(SEQUENCER_ADDRESS).into_()],
    );
}

#[test]
fn cheat_sequencer_address_library_call() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("CheatSequencerAddressChecker", &contracts_data);

    let lib_call_address = test_env.deploy("CheatSequencerAddressCheckerLibCall", &[]);
    let lib_call_selector = "get_sequencer_address_with_lib_call";

    test_env.start_cheat_sequencer_address(lib_call_address, 123);

    assert_success(
        test_env.call_contract(&lib_call_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt252::from(123)],
    );

    test_env.stop_cheat_sequencer_address(lib_call_address);

    assert_success(
        test_env.call_contract(&lib_call_address, lib_call_selector, &[class_hash.into_()]),
        &[contract_address!(SEQUENCER_ADDRESS).into_()],
    );
}

#[test]
fn cheat_sequencer_address_all_simple() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatSequencerAddressChecker", &[]);

    test_env.cheatnet_state.cheat_sequencer_address_global(123_u8.into());

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[Felt252::from(123)],
    );
}

#[test]
fn cheat_sequencer_address_all_then_one() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatSequencerAddressChecker", &[]);

    test_env.cheatnet_state.cheat_sequencer_address_global(111_u8.into());

    test_env.start_cheat_sequencer_address(contract_address, 222);

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[Felt252::from(222)],
    );
}

#[test]
fn cheat_sequencer_address_one_then_all() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatSequencerAddressChecker", &[]);

    test_env.start_cheat_sequencer_address(contract_address, 111);
    test_env.cheatnet_state.cheat_sequencer_address_global(222_u8.into());

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[Felt252::from(222)],
    );
}

#[test]
fn cheat_sequencer_address_all_stop() {
    let mut test_env = TestEnvironment::new();

    let cheat_sequencer_address_checker = test_env.declare("CheatSequencerAddressChecker", &get_contracts());
    let contract_address = test_env.deploy_wrapper(&cheat_sequencer_address_checker, &[]);

    test_env.cheatnet_state.cheat_sequencer_address_global(123_u8.into());

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[Felt252::from(123)],
    );

    test_env.cheatnet_state.stop_cheat_sequencer_address_global();

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[Felt252::from(123)],
    );

    let contract_address = test_env.deploy_wrapper(&cheat_sequencer_address_checker, &[]);

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[contract_address!(SEQUENCER_ADDRESS).into_()],
    );
}

#[test]
fn cheat_sequencer_address_multiple() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("CheatSequencerAddressChecker", &contracts_data);

    let contract_address1 = test_env.deploy_wrapper(&class_hash, &[]);
    let contract_address2 = test_env.deploy_wrapper(&class_hash, &[]);

    test_env.start_cheat_sequencer_address(contract_address1, 123);
    test_env.start_cheat_sequencer_address(contract_address2, 123);

    assert_success(
        test_env.call_contract(&contract_address1, "get_sequencer_address", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address2, "get_sequencer_address", &[]),
        &[Felt252::from(123)],
    );

    test_env.cheatnet_state.stop_cheat_sequencer_address(contract_address1);
    test_env.cheatnet_state.stop_cheat_sequencer_address(contract_address2);

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
fn cheat_sequencer_address_simple_with_span() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatSequencerAddressChecker", &[]);

    test_env.cheat_sequencer_address(contract_address, 123, CheatSpan::TargetCalls(2));

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
fn cheat_sequencer_address_proxy_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("CheatSequencerAddressCheckerProxy", &contracts_data);
    let contract_address_1 = test_env.deploy_wrapper(&class_hash, &[]);
    let contract_address_2 = test_env.deploy_wrapper(&class_hash, &[]);

    test_env.cheat_sequencer_address(contract_address_1, 123, CheatSpan::TargetCalls(1));

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
fn cheat_sequencer_address_in_constructor_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();

    let class_hash = test_env.declare("ConstructorCheatSequencerAddressChecker", &contracts_data);
    let precalculated_address = test_env
        .cheatnet_state
        .precalculate_address(&class_hash, &[]);

    test_env.cheat_sequencer_address(precalculated_address, 123, CheatSpan::TargetCalls(2));

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
fn cheat_sequencer_address_no_constructor_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();

    let class_hash = test_env.declare("CheatSequencerAddressChecker", &contracts_data);
    let precalculated_address = test_env
        .cheatnet_state
        .precalculate_address(&class_hash, &[]);

    test_env.cheat_sequencer_address(precalculated_address, 123, CheatSpan::TargetCalls(1));

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
fn cheat_sequencer_address_override_span() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatSequencerAddressChecker", &[]);

    test_env.cheat_sequencer_address(contract_address, 123, CheatSpan::TargetCalls(2));

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[Felt252::from(123)],
    );

    test_env.cheat_sequencer_address(contract_address, 321, CheatSpan::Indefinite);

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[Felt252::from(321)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[Felt252::from(321)],
    );

    test_env.stop_cheat_sequencer_address(contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_sequencer_address", &[]),
        &[contract_address!(SEQUENCER_ADDRESS).into_()],
    );
}

#[test]
fn cheat_sequencer_address_library_call_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("CheatSequencerAddressChecker", &contracts_data);
    let contract_address = test_env.deploy("CheatSequencerAddressCheckerLibCall", &[]);

    test_env.cheat_sequencer_address(contract_address, 123, CheatSpan::TargetCalls(1));

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
