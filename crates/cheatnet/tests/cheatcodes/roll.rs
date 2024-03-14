use crate::common::assertions::assert_success;
use crate::common::get_contracts;
use cairo_felt::Felt252;
use cheatnet::state::{CheatSpan, CheatTarget, CheatnetState};
use conversions::IntoConv;
use runtime::starknet::context::DEFAULT_BLOCK_NUMBER;
use starknet_api::core::ContractAddress;

use super::test_environment::TestEnvironment;

trait RollTrait {
    fn roll(&mut self, target: CheatTarget, block_number: u128, span: CheatSpan);
    fn start_roll(&mut self, target: CheatTarget, block_number: u128);
    fn stop_roll(&mut self, contract_address: &ContractAddress);
}

impl<'a> RollTrait for TestEnvironment<'a> {
    fn roll(&mut self, target: CheatTarget, block_number: u128, span: CheatSpan) {
        self.runtime_state
            .cheatnet_state
            .roll(target, Felt252::from(block_number), span);
    }

    fn start_roll(&mut self, target: CheatTarget, block_number: u128) {
        self.runtime_state
            .cheatnet_state
            .start_roll(target, Felt252::from(block_number));
    }

    fn stop_roll(&mut self, contract_address: &ContractAddress) {
        self.runtime_state
            .cheatnet_state
            .stop_roll(CheatTarget::One(*contract_address));
    }
}

#[test]
fn roll_simple() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address = test_env.deploy("RollChecker", &[]);

    test_env.start_roll(CheatTarget::One(contract_address), 123);

    let output = test_env.call_contract(&contract_address, "get_block_number", &[]);
    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn roll_with_other_syscall() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address = test_env.deploy("RollChecker", &[]);

    test_env.start_roll(CheatTarget::One(contract_address), 123);

    let output = test_env.call_contract(&contract_address, "get_block_number_and_emit_event", &[]);
    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn roll_in_constructor() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);
    let contracts = get_contracts();

    let class_hash = test_env.declare("ConstructorRollChecker", &contracts);
    let precalculated_address = test_env.precalculate_address(&class_hash, &[]);

    test_env.start_roll(CheatTarget::One(precalculated_address), 123);

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);
    assert_eq!(precalculated_address, contract_address);

    let output = test_env.call_contract(&contract_address, "get_stored_block_number", &[]);
    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn roll_stop() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address = test_env.deploy("RollChecker", &[]);

    test_env.start_roll(CheatTarget::One(contract_address), 123);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(123)],
    );

    test_env.stop_roll(&contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(DEFAULT_BLOCK_NUMBER)],
    );
}

#[test]
fn roll_double() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address = test_env.deploy("RollChecker", &[]);

    test_env.start_roll(CheatTarget::One(contract_address), 123);
    test_env.start_roll(CheatTarget::One(contract_address), 123);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(123)],
    );

    test_env.stop_roll(&contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(DEFAULT_BLOCK_NUMBER)],
    );
}

#[test]
fn roll_proxy() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address = test_env.deploy("RollChecker", &[]);
    let proxy_address = test_env.deploy("RollCheckerProxy", &[]);

    let proxy_selector = "get_roll_checkers_block_number";

    test_env.start_roll(CheatTarget::One(contract_address), 123);

    assert_success(
        test_env.call_contract(&proxy_address, proxy_selector, &[contract_address.into_()]),
        &[Felt252::from(123)],
    );

    test_env.stop_roll(&contract_address);

    assert_success(
        test_env.call_contract(&proxy_address, proxy_selector, &[contract_address.into_()]),
        &[Felt252::from(DEFAULT_BLOCK_NUMBER)],
    );
}

#[test]
fn roll_library_call() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contracts = get_contracts();
    let class_hash = test_env.declare("RollChecker", &contracts);

    let lib_call_address = test_env.deploy("RollCheckerLibCall", &[]);
    let lib_call_selector = "get_block_number_with_lib_call";

    test_env.start_roll(CheatTarget::One(lib_call_address), 123);

    assert_success(
        test_env.call_contract(&lib_call_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt252::from(123)],
    );
    test_env.stop_roll(&lib_call_address);

    assert_success(
        test_env.call_contract(&lib_call_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt252::from(DEFAULT_BLOCK_NUMBER)],
    );
}

#[test]
fn roll_all_simple() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address = test_env.deploy("RollChecker", &[]);

    test_env.start_roll(CheatTarget::All, 123);

    let output = test_env.call_contract(&contract_address, "get_block_number", &[]);
    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn roll_all_then_one() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address = test_env.deploy("RollChecker", &[]);

    test_env.start_roll(CheatTarget::All, 321);
    test_env.start_roll(CheatTarget::One(contract_address), 123);

    let output = test_env.call_contract(&contract_address, "get_block_number", &[]);
    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn roll_one_then_all() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address = test_env.deploy("RollChecker", &[]);

    test_env.start_roll(CheatTarget::One(contract_address), 123);
    test_env.start_roll(CheatTarget::All, 321);

    let output = test_env.call_contract(&contract_address, "get_block_number", &[]);
    assert_success(output, &[Felt252::from(321)]);
}

#[test]
fn roll_all_stop() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address = test_env.deploy("RollChecker", &[]);

    test_env.start_roll(CheatTarget::All, 123);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(123)],
    );

    test_env
        .runtime_state
        .cheatnet_state
        .stop_roll(CheatTarget::All);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(DEFAULT_BLOCK_NUMBER)],
    );
}

#[test]
fn roll_multiple() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contracts = get_contracts();
    let class_hash = test_env.declare("RollChecker", &contracts);

    let contract_address1 = test_env.deploy_wrapper(&class_hash, &[]);
    let contract_address2 = test_env.deploy_wrapper(&class_hash, &[]);

    test_env.runtime_state.cheatnet_state.start_roll(
        CheatTarget::Multiple(vec![contract_address1, contract_address2]),
        Felt252::from(123),
    );

    assert_success(
        test_env.call_contract(&contract_address1, "get_block_number", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address2, "get_block_number", &[]),
        &[Felt252::from(123)],
    );

    test_env
        .runtime_state
        .cheatnet_state
        .stop_roll(CheatTarget::Multiple(vec![
            contract_address1,
            contract_address2,
        ]));

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
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address = test_env.deploy("RollChecker", &[]);

    test_env.roll(
        CheatTarget::One(contract_address),
        123,
        CheatSpan::Number(2),
    );

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
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contracts = get_contracts();
    let class_hash = test_env.declare("RollCheckerProxy", &contracts);
    let contract_address_1 = test_env.deploy_wrapper(&class_hash, &[]);
    let contract_address_2 = test_env.deploy_wrapper(&class_hash, &[]);

    test_env.roll(
        CheatTarget::One(contract_address_1),
        123,
        CheatSpan::Number(1),
    );

    let output = test_env.call_contract(
        &contract_address_1,
        "call_proxy",
        &[contract_address_2.into_()],
    );
    assert_success(output, &[123.into(), DEFAULT_BLOCK_NUMBER.into()]);
}

#[test]
fn roll_in_constructor_with_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contracts = get_contracts();

    let class_hash = test_env.declare("ConstructorRollChecker", &contracts);
    let precalculated_address = test_env.precalculate_address(&class_hash, &[]);

    test_env.roll(
        CheatTarget::One(precalculated_address),
        123,
        CheatSpan::Number(2),
    );

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
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contracts = get_contracts();

    let class_hash = test_env.declare("RollChecker", &contracts);
    let precalculated_address = test_env.precalculate_address(&class_hash, &[]);

    test_env.roll(
        CheatTarget::One(precalculated_address),
        123,
        CheatSpan::Number(1),
    );

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
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address = test_env.deploy("RollChecker", &[]);

    test_env.roll(
        CheatTarget::One(contract_address),
        123,
        CheatSpan::Number(2),
    );

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(123)],
    );

    test_env.roll(
        CheatTarget::One(contract_address),
        321,
        CheatSpan::Indefinite,
    );

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(321)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(321)],
    );

    test_env.stop_roll(&contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(DEFAULT_BLOCK_NUMBER)],
    );
}

#[test]
fn roll_library_call_with_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contracts = get_contracts();
    let class_hash = test_env.declare("RollChecker", &contracts);
    let contract_address = test_env.deploy("RollCheckerLibCall", &[]);

    test_env.roll(
        CheatTarget::One(contract_address),
        123,
        CheatSpan::Number(1),
    );

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

#[test]
fn roll_all_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address_1 = test_env.deploy("RollChecker", &[]);
    let contract_address_2 = test_env.deploy("RollCheckerLibCall", &[]);

    test_env.roll(CheatTarget::All, 123, CheatSpan::Number(1));

    assert_success(
        test_env.call_contract(&contract_address_1, "get_block_number", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address_1, "get_block_number", &[]),
        &[Felt252::from(DEFAULT_BLOCK_NUMBER)],
    );

    assert_success(
        test_env.call_contract(&contract_address_2, "get_block_number", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address_2, "get_block_number", &[]),
        &[Felt252::from(DEFAULT_BLOCK_NUMBER)],
    );
}
