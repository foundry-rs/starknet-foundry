use crate::{common::assertions::assert_success, common::get_contracts};
use cairo_felt::Felt252;
use cheatnet::state::CheatSpan;
use conversions::IntoConv;
use runtime::starknet::context::DEFAULT_BLOCK_NUMBER;
use starknet_api::core::ContractAddress;

use super::test_environment::TestEnvironment;

trait RollTrait {
    fn roll(&mut self, contract_address: ContractAddress, block_number: u64, span: CheatSpan);
    fn start_roll(&mut self, contract_address: ContractAddress, block_number: u64);
    fn stop_roll(&mut self, contract_address: ContractAddress);
}

impl RollTrait for TestEnvironment {
    fn roll(&mut self, contract_address: ContractAddress, block_number: u64, span: CheatSpan) {
        self.cheatnet_state
            .roll(contract_address, block_number, span);
    }

    fn start_roll(&mut self, contract_address: ContractAddress, block_number: u64) {
        self.cheatnet_state
            .start_roll(contract_address, block_number);
    }

    fn stop_roll(&mut self, contract_address: ContractAddress) {
        self.cheatnet_state.stop_roll(contract_address);
    }
}

#[test]
fn roll_simple() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("RollChecker", &[]);

    test_env.start_roll(contract_address, 123);

    let output = test_env.call_contract(&contract_address, "get_block_number", &[]);
    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn roll_with_other_syscall() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("RollChecker", &[]);

    test_env.start_roll(contract_address, 123);

    let output = test_env.call_contract(&contract_address, "get_block_number_and_emit_event", &[]);
    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn roll_in_constructor() {
    let mut test_env = TestEnvironment::new();
    let contracts_data = get_contracts();

    let class_hash = test_env.declare("ConstructorRollChecker", &contracts_data);
    let precalculated_address = test_env.precalculate_address(&class_hash, &[]);

    test_env.start_roll(precalculated_address, 123);

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);
    assert_eq!(precalculated_address, contract_address);

    let output = test_env.call_contract(&contract_address, "get_stored_block_number", &[]);
    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn roll_stop() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("RollChecker", &[]);

    test_env.start_roll(contract_address, 123);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(123)],
    );

    test_env.stop_roll(contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(DEFAULT_BLOCK_NUMBER)],
    );
}

#[test]
fn roll_double() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("RollChecker", &[]);

    test_env.start_roll(contract_address, 123);
    test_env.start_roll(contract_address, 123);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(123)],
    );

    test_env.stop_roll(contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(DEFAULT_BLOCK_NUMBER)],
    );
}

#[test]
fn roll_proxy() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("RollChecker", &[]);
    let proxy_address = test_env.deploy("RollCheckerProxy", &[]);

    let proxy_selector = "get_roll_checkers_block_number";

    test_env.start_roll(contract_address, 123);

    assert_success(
        test_env.call_contract(&proxy_address, proxy_selector, &[contract_address.into_()]),
        &[Felt252::from(123)],
    );

    test_env.stop_roll(contract_address);

    assert_success(
        test_env.call_contract(&proxy_address, proxy_selector, &[contract_address.into_()]),
        &[Felt252::from(DEFAULT_BLOCK_NUMBER)],
    );
}

#[test]
fn roll_library_call() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("RollChecker", &contracts_data);

    let lib_call_address = test_env.deploy("RollCheckerLibCall", &[]);
    let lib_call_selector = "get_block_number_with_lib_call";

    test_env.start_roll(lib_call_address, 123);

    assert_success(
        test_env.call_contract(&lib_call_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt252::from(123)],
    );
    test_env.stop_roll(lib_call_address);

    assert_success(
        test_env.call_contract(&lib_call_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt252::from(DEFAULT_BLOCK_NUMBER)],
    );
}

#[test]
fn roll_all_simple() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("RollChecker", &[]);

    test_env.cheatnet_state.roll_global(123);

    let output = test_env.call_contract(&contract_address, "get_block_number", &[]);
    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn roll_all_then_one() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("RollChecker", &[]);

    test_env.cheatnet_state.roll_global(321);
    test_env.start_roll(contract_address, 123);

    let output = test_env.call_contract(&contract_address, "get_block_number", &[]);
    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn roll_one_then_all() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("RollChecker", &[]);

    test_env.start_roll(contract_address, 123);
    test_env.cheatnet_state.roll_global(321);

    let output = test_env.call_contract(&contract_address, "get_block_number", &[]);
    assert_success(output, &[Felt252::from(321)]);
}

#[test]
fn roll_all_stop() {
    let mut test_env = TestEnvironment::new();

    let roll_checker = test_env.declare("RollChecker", &get_contracts());
    let contract_address = test_env.deploy_wrapper(&roll_checker, &[]);

    test_env.cheatnet_state.roll_global(123);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(123)],
    );

    test_env.cheatnet_state.stop_roll_global();

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(123)],
    );

    let contract_address = test_env.deploy_wrapper(&roll_checker, &[]);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(DEFAULT_BLOCK_NUMBER)],
    );
}

#[test]
fn roll_multiple() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("RollChecker", &contracts_data);

    let contract_address1 = test_env.deploy_wrapper(&class_hash, &[]);
    let contract_address2 = test_env.deploy_wrapper(&class_hash, &[]);

    test_env.cheatnet_state.start_roll(contract_address1, 123);
    test_env.cheatnet_state.start_roll(contract_address2, 123);

    assert_success(
        test_env.call_contract(&contract_address1, "get_block_number", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address2, "get_block_number", &[]),
        &[Felt252::from(123)],
    );

    test_env.cheatnet_state.stop_roll(contract_address1);
    test_env.cheatnet_state.stop_roll(contract_address2);

    assert_success(
        test_env.call_contract(&contract_address1, "get_block_number", &[]),
        &[Felt252::from(DEFAULT_BLOCK_NUMBER)],
    );
    assert_success(
        test_env.call_contract(&contract_address2, "get_block_number", &[]),
        &[Felt252::from(DEFAULT_BLOCK_NUMBER)],
    );
}

#[test]
fn roll_simple_with_span() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("RollChecker", &[]);

    test_env.roll(contract_address, 123, CheatSpan::TargetCalls(2));

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(DEFAULT_BLOCK_NUMBER)],
    );
}

#[test]
fn roll_proxy_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("RollCheckerProxy", &contracts_data);
    let contract_address_1 = test_env.deploy_wrapper(&class_hash, &[]);
    let contract_address_2 = test_env.deploy_wrapper(&class_hash, &[]);

    test_env.roll(contract_address_1, 123, CheatSpan::TargetCalls(1));

    let output = test_env.call_contract(
        &contract_address_1,
        "call_proxy",
        &[contract_address_2.into_()],
    );
    assert_success(output, &[123.into(), DEFAULT_BLOCK_NUMBER.into()]);
}

#[test]
fn roll_in_constructor_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();

    let class_hash = test_env.declare("ConstructorRollChecker", &contracts_data);
    let precalculated_address = test_env.precalculate_address(&class_hash, &[]);

    test_env.roll(precalculated_address, 123, CheatSpan::TargetCalls(2));

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);
    assert_eq!(precalculated_address, contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(DEFAULT_BLOCK_NUMBER)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_stored_block_number", &[]),
        &[Felt252::from(123)],
    );
}

#[test]
fn roll_no_constructor_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();

    let class_hash = test_env.declare("RollChecker", &contracts_data);
    let precalculated_address = test_env.precalculate_address(&class_hash, &[]);

    test_env.roll(precalculated_address, 123, CheatSpan::TargetCalls(1));

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);
    assert_eq!(precalculated_address, contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(DEFAULT_BLOCK_NUMBER)],
    );
}

#[test]
fn roll_override_span() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("RollChecker", &[]);

    test_env.roll(contract_address, 123, CheatSpan::TargetCalls(2));

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(123)],
    );

    test_env.roll(contract_address, 321, CheatSpan::Indefinite);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(321)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(321)],
    );

    test_env.stop_roll(contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(DEFAULT_BLOCK_NUMBER)],
    );
}

#[test]
fn roll_library_call_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("RollChecker", &contracts_data);
    let contract_address = test_env.deploy("RollCheckerLibCall", &[]);

    test_env.roll(contract_address, 123, CheatSpan::TargetCalls(1));

    let lib_call_selector = "get_block_number_with_lib_call";

    assert_success(
        test_env.call_contract(&contract_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt252::from(DEFAULT_BLOCK_NUMBER)],
    );
}
