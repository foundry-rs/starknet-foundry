use crate::{common::assertions::assert_success, common::get_contracts};
use cairo_felt::Felt252;
use cheatnet::state::{CheatSpan, CheatTarget};
use conversions::IntoConv;
use starknet_api::core::ContractAddress;

use super::test_environment::TestEnvironment;

const DEFAULT_BLOCK_TIMESTAMP: u64 = 0;

trait WarpTrait {
    fn warp(&mut self, target: CheatTarget, timestamp: u128, span: CheatSpan);
    fn start_warp(&mut self, target: CheatTarget, timestamp: u128);
    fn stop_warp(&mut self, contract_address: &ContractAddress);
}

impl WarpTrait for TestEnvironment {
    fn warp(&mut self, target: CheatTarget, timestamp: u128, span: CheatSpan) {
        self.cheatnet_state
            .warp(target, Felt252::from(timestamp), span);
    }

    fn start_warp(&mut self, target: CheatTarget, timestamp: u128) {
        self.cheatnet_state
            .start_warp(target, Felt252::from(timestamp));
    }

    fn stop_warp(&mut self, contract_address: &ContractAddress) {
        self.cheatnet_state
            .stop_warp(CheatTarget::One(*contract_address));
    }
}

#[test]
fn warp_simple() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("WarpChecker", &[]);

    test_env.start_warp(CheatTarget::One(contract_address), 123);

    let output = test_env.call_contract(&contract_address, "get_block_timestamp", &[]);
    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn warp_with_other_syscall() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("WarpChecker", &[]);

    test_env.start_warp(CheatTarget::One(contract_address), 123);
    let selector = "get_block_timestamp_and_emit_event";

    let output = test_env.call_contract(&contract_address, selector, &[]);
    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn warp_in_constructor() {
    let mut test_env = TestEnvironment::new();
    let contracts_data = get_contracts();

    let class_hash = test_env.declare("ConstructorWarpChecker", &contracts_data);
    let precalculated_address = test_env.precalculate_address(&class_hash, &[]);

    test_env.start_warp(CheatTarget::One(precalculated_address), 123);

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);

    assert_eq!(precalculated_address, contract_address);

    let output = test_env.call_contract(&contract_address, "get_stored_block_timestamp", &[]);
    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn warp_stop() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("WarpChecker", &[]);

    test_env.start_warp(CheatTarget::One(contract_address), 123);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt252::from(123)],
    );

    test_env.stop_warp(&contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt252::from(DEFAULT_BLOCK_TIMESTAMP)],
    );
}

#[test]
fn warp_double() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("WarpChecker", &[]);

    test_env.start_warp(CheatTarget::One(contract_address), 123);
    test_env.start_warp(CheatTarget::One(contract_address), 123);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt252::from(123)],
    );

    test_env.stop_warp(&contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt252::from(DEFAULT_BLOCK_TIMESTAMP)],
    );
}

#[test]
fn warp_proxy() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("WarpChecker", &[]);
    let proxy_address = test_env.deploy("WarpCheckerProxy", &[]);
    let proxy_selector = "get_warp_checkers_block_timestamp";

    test_env.start_warp(CheatTarget::One(contract_address), 123);

    assert_success(
        test_env.call_contract(&proxy_address, proxy_selector, &[contract_address.into_()]),
        &[Felt252::from(123)],
    );

    test_env.stop_warp(&contract_address);

    assert_success(
        test_env.call_contract(&proxy_address, proxy_selector, &[contract_address.into_()]),
        &[Felt252::from(DEFAULT_BLOCK_TIMESTAMP)],
    );
}

#[test]
fn warp_library_call() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("WarpChecker", &contracts_data);

    let lib_call_address = test_env.deploy("WarpCheckerLibCall", &[]);
    let lib_call_selector = "get_block_timestamp_with_lib_call";

    test_env.start_warp(CheatTarget::One(lib_call_address), 123);

    assert_success(
        test_env.call_contract(&lib_call_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt252::from(123)],
    );
    test_env.stop_warp(&lib_call_address);

    assert_success(
        test_env.call_contract(&lib_call_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt252::from(DEFAULT_BLOCK_TIMESTAMP)],
    );
}

#[test]
fn warp_all_simple() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("WarpChecker", &[]);

    test_env.start_warp(CheatTarget::All, 123);

    let output = test_env.call_contract(&contract_address, "get_block_timestamp", &[]);
    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn warp_all_then_one() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("WarpChecker", &[]);

    test_env.start_warp(CheatTarget::All, 321);
    test_env.start_warp(CheatTarget::One(contract_address), 123);

    let output = test_env.call_contract(&contract_address, "get_block_timestamp", &[]);
    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn warp_one_then_all() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("WarpChecker", &[]);

    test_env.start_warp(CheatTarget::One(contract_address), 123);
    test_env.start_warp(CheatTarget::All, 321);

    let output = test_env.call_contract(&contract_address, "get_block_timestamp", &[]);
    assert_success(output, &[Felt252::from(321)]);
}

#[test]
fn warp_all_stop() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("WarpChecker", &[]);

    test_env.start_warp(CheatTarget::All, 123);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt252::from(123)],
    );

    test_env.cheatnet_state.stop_warp(CheatTarget::All);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt252::from(DEFAULT_BLOCK_TIMESTAMP)],
    );
}

#[test]
fn warp_multiple() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("WarpChecker", &contracts_data);

    let contract_address1 = test_env.deploy_wrapper(&class_hash, &[]);
    let contract_address2 = test_env.deploy_wrapper(&class_hash, &[]);

    test_env.cheatnet_state.start_warp(
        CheatTarget::Multiple(vec![contract_address1, contract_address2]),
        Felt252::from(123),
    );

    assert_success(
        test_env.call_contract(&contract_address1, "get_block_timestamp", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address2, "get_block_timestamp", &[]),
        &[Felt252::from(123)],
    );

    test_env
        .cheatnet_state
        .stop_warp(CheatTarget::Multiple(vec![
            contract_address1,
            contract_address2,
        ]));

    assert_success(
        test_env.call_contract(&contract_address1, "get_block_timestamp", &[]),
        &[Felt252::from(DEFAULT_BLOCK_TIMESTAMP)],
    );
    assert_success(
        test_env.call_contract(&contract_address2, "get_block_timestamp", &[]),
        &[Felt252::from(DEFAULT_BLOCK_TIMESTAMP)],
    );
}

#[test]
fn warp_simple_with_span() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("WarpChecker", &[]);

    test_env.warp(
        CheatTarget::One(contract_address),
        123,
        CheatSpan::TargetCalls(2),
    );

    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt252::from(DEFAULT_BLOCK_TIMESTAMP)],
    );
}

#[test]
fn warp_proxy_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("WarpCheckerProxy", &contracts_data);
    let contract_address_1 = test_env.deploy_wrapper(&class_hash, &[]);
    let contract_address_2 = test_env.deploy_wrapper(&class_hash, &[]);

    test_env.warp(
        CheatTarget::One(contract_address_1),
        123,
        CheatSpan::TargetCalls(1),
    );

    let output = test_env.call_contract(
        &contract_address_1,
        "call_proxy",
        &[contract_address_2.into_()],
    );
    assert_success(output, &[123.into(), DEFAULT_BLOCK_TIMESTAMP.into()]);
}

#[test]
fn warp_in_constructor_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();

    let class_hash = test_env.declare("ConstructorWarpChecker", &contracts_data);
    let precalculated_address = test_env.precalculate_address(&class_hash, &[]);

    test_env.warp(
        CheatTarget::One(precalculated_address),
        123,
        CheatSpan::TargetCalls(2),
    );

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);
    assert_eq!(precalculated_address, contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt252::from(DEFAULT_BLOCK_TIMESTAMP)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_stored_block_timestamp", &[]),
        &[Felt252::from(123)],
    );
}

#[test]
fn warp_no_constructor_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();

    let class_hash = test_env.declare("WarpChecker", &contracts_data);
    let precalculated_address = test_env.precalculate_address(&class_hash, &[]);

    test_env.warp(
        CheatTarget::One(precalculated_address),
        123,
        CheatSpan::TargetCalls(1),
    );

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);
    assert_eq!(precalculated_address, contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt252::from(DEFAULT_BLOCK_TIMESTAMP)],
    );
}

#[test]
fn warp_override_span() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("WarpChecker", &[]);

    test_env.warp(
        CheatTarget::One(contract_address),
        123,
        CheatSpan::TargetCalls(2),
    );

    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt252::from(123)],
    );

    test_env.warp(
        CheatTarget::One(contract_address),
        321,
        CheatSpan::Indefinite,
    );

    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt252::from(321)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt252::from(321)],
    );

    test_env.stop_warp(&contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt252::from(DEFAULT_BLOCK_TIMESTAMP)],
    );
}

#[test]
fn warp_library_call_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("WarpChecker", &contracts_data);
    let contract_address = test_env.deploy("WarpCheckerLibCall", &[]);

    test_env.warp(
        CheatTarget::One(contract_address),
        123,
        CheatSpan::TargetCalls(1),
    );

    let lib_call_selector = "get_block_timestamp_with_lib_call";

    assert_success(
        test_env.call_contract(&contract_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt252::from(DEFAULT_BLOCK_TIMESTAMP)],
    );
}

#[test]
fn warp_all_span() {
    let mut test_env = TestEnvironment::new();

    let contract_address_1 = test_env.deploy("WarpChecker", &[]);
    let contract_address_2 = test_env.deploy("WarpCheckerLibCall", &[]);

    test_env.warp(CheatTarget::All, 123, CheatSpan::TargetCalls(1));

    assert_success(
        test_env.call_contract(&contract_address_1, "get_block_timestamp", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address_1, "get_block_timestamp", &[]),
        &[Felt252::from(DEFAULT_BLOCK_TIMESTAMP)],
    );

    assert_success(
        test_env.call_contract(&contract_address_2, "get_block_timestamp", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address_2, "get_block_timestamp", &[]),
        &[Felt252::from(DEFAULT_BLOCK_TIMESTAMP)],
    );
}
