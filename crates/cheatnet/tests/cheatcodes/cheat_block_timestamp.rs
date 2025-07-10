use crate::{common::assertions::assert_success, common::get_contracts};
use cheatnet::state::CheatSpan;
use conversions::IntoConv;
use starknet_api::core::ContractAddress;
use starknet_types_core::felt::Felt;
use std::num::NonZeroUsize;

use super::test_environment::TestEnvironment;

const DEFAULT_BLOCK_TIMESTAMP: u64 = 0;

trait CheatBlockTimestampTrait {
    fn cheat_block_timestamp(
        &mut self,
        contract_address: ContractAddress,
        timestamp: u64,
        span: CheatSpan,
    );
    fn start_cheat_block_timestamp(&mut self, contract_address: ContractAddress, timestamp: u64);
    fn stop_cheat_block_timestamp(&mut self, contract_address: ContractAddress);
}

impl CheatBlockTimestampTrait for TestEnvironment {
    fn cheat_block_timestamp(
        &mut self,
        contract_address: ContractAddress,
        timestamp: u64,
        span: CheatSpan,
    ) {
        self.cheatnet_state
            .cheat_block_timestamp(contract_address, timestamp, span);
    }

    fn start_cheat_block_timestamp(&mut self, contract_address: ContractAddress, timestamp: u64) {
        self.cheatnet_state
            .start_cheat_block_timestamp(contract_address, timestamp);
    }

    fn stop_cheat_block_timestamp(&mut self, contract_address: ContractAddress) {
        self.cheatnet_state
            .stop_cheat_block_timestamp(contract_address);
    }
}

#[test]
fn cheat_block_timestamp_simple() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockTimestampChecker", &[]);

    test_env.start_cheat_block_timestamp(contract_address, 123);

    let output = test_env.call_contract(&contract_address, "get_block_timestamp", &[]);
    assert_success(output, &[Felt::from(123)]);
}

#[test]
fn cheat_block_timestamp_with_other_syscall() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockTimestampChecker", &[]);

    test_env.start_cheat_block_timestamp(contract_address, 123);
    let selector = "get_block_timestamp_and_emit_event";

    let output = test_env.call_contract(&contract_address, selector, &[]);
    assert_success(output, &[Felt::from(123)]);
}

#[test]
fn cheat_block_timestamp_in_constructor() {
    let mut test_env = TestEnvironment::new();
    let contracts_data = get_contracts();

    let class_hash = test_env.declare("ConstructorCheatBlockTimestampChecker", &contracts_data);
    let precalculated_address = test_env.precalculate_address(&class_hash, &[]);

    test_env.start_cheat_block_timestamp(precalculated_address, 123);

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);

    assert_eq!(precalculated_address, contract_address);

    let output = test_env.call_contract(&contract_address, "get_stored_block_timestamp", &[]);
    assert_success(output, &[Felt::from(123)]);
}

#[test]
fn cheat_block_timestamp_stop() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockTimestampChecker", &[]);

    test_env.start_cheat_block_timestamp(contract_address, 123);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt::from(123)],
    );

    test_env.stop_cheat_block_timestamp(contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt::from(DEFAULT_BLOCK_TIMESTAMP)],
    );
}

#[test]
fn cheat_block_timestamp_double() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockTimestampChecker", &[]);

    test_env.start_cheat_block_timestamp(contract_address, 123);
    test_env.start_cheat_block_timestamp(contract_address, 123);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt::from(123)],
    );

    test_env.stop_cheat_block_timestamp(contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt::from(DEFAULT_BLOCK_TIMESTAMP)],
    );
}

#[test]
fn cheat_block_timestamp_proxy() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockTimestampChecker", &[]);
    let proxy_address = test_env.deploy("CheatBlockTimestampCheckerProxy", &[]);
    let proxy_selector = "get_cheated_block_timestamp";

    test_env.start_cheat_block_timestamp(contract_address, 123);

    assert_success(
        test_env.call_contract(&proxy_address, proxy_selector, &[contract_address.into_()]),
        &[Felt::from(123)],
    );

    test_env.stop_cheat_block_timestamp(contract_address);

    assert_success(
        test_env.call_contract(&proxy_address, proxy_selector, &[contract_address.into_()]),
        &[Felt::from(DEFAULT_BLOCK_TIMESTAMP)],
    );
}

#[test]
fn cheat_block_timestamp_library_call() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("CheatBlockTimestampChecker", &contracts_data);

    let lib_call_address = test_env.deploy("CheatBlockTimestampCheckerLibCall", &[]);
    let lib_call_selector = "get_block_timestamp_with_lib_call";

    test_env.start_cheat_block_timestamp(lib_call_address, 123);

    assert_success(
        test_env.call_contract(&lib_call_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt::from(123)],
    );
    test_env.stop_cheat_block_timestamp(lib_call_address);

    assert_success(
        test_env.call_contract(&lib_call_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt::from(DEFAULT_BLOCK_TIMESTAMP)],
    );
}

#[test]
fn cheat_block_timestamp_all_simple() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockTimestampChecker", &[]);

    test_env
        .cheatnet_state
        .start_cheat_block_timestamp_global(123);

    let output = test_env.call_contract(&contract_address, "get_block_timestamp", &[]);
    assert_success(output, &[Felt::from(123)]);
}

#[test]
fn cheat_block_timestamp_all_then_one() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockTimestampChecker", &[]);

    test_env
        .cheatnet_state
        .start_cheat_block_timestamp_global(321);

    test_env.start_cheat_block_timestamp(contract_address, 123);

    let output = test_env.call_contract(&contract_address, "get_block_timestamp", &[]);
    assert_success(output, &[Felt::from(123)]);
}

#[test]
fn cheat_block_timestamp_one_then_all() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockTimestampChecker", &[]);

    test_env.start_cheat_block_timestamp(contract_address, 123);

    test_env
        .cheatnet_state
        .start_cheat_block_timestamp_global(321);

    let output = test_env.call_contract(&contract_address, "get_block_timestamp", &[]);
    assert_success(output, &[Felt::from(321)]);
}

#[test]
fn cheat_block_timestamp_all_stop() {
    let mut test_env = TestEnvironment::new();

    let cheat_block_timestamp_checker =
        test_env.declare("CheatBlockTimestampChecker", &get_contracts());

    let contract_address = test_env.deploy_wrapper(&cheat_block_timestamp_checker, &[]);

    test_env
        .cheatnet_state
        .start_cheat_block_timestamp_global(123);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt::from(123)],
    );

    test_env.cheatnet_state.stop_cheat_block_timestamp_global();

    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt::from(DEFAULT_BLOCK_TIMESTAMP)],
    );

    let contract_address = test_env.deploy_wrapper(&cheat_block_timestamp_checker, &[]);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt::from(DEFAULT_BLOCK_TIMESTAMP)],
    );
}

#[test]
fn cheat_block_timestamp_multiple() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("CheatBlockTimestampChecker", &contracts_data);

    let contract_address1 = test_env.deploy_wrapper(&class_hash, &[]);
    let contract_address2 = test_env.deploy_wrapper(&class_hash, &[]);

    test_env
        .cheatnet_state
        .start_cheat_block_timestamp(contract_address1, 123);
    test_env
        .cheatnet_state
        .start_cheat_block_timestamp(contract_address2, 123);

    assert_success(
        test_env.call_contract(&contract_address1, "get_block_timestamp", &[]),
        &[Felt::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address2, "get_block_timestamp", &[]),
        &[Felt::from(123)],
    );

    test_env
        .cheatnet_state
        .stop_cheat_block_timestamp(contract_address1);
    test_env
        .cheatnet_state
        .stop_cheat_block_timestamp(contract_address2);

    assert_success(
        test_env.call_contract(&contract_address1, "get_block_timestamp", &[]),
        &[Felt::from(DEFAULT_BLOCK_TIMESTAMP)],
    );
    assert_success(
        test_env.call_contract(&contract_address2, "get_block_timestamp", &[]),
        &[Felt::from(DEFAULT_BLOCK_TIMESTAMP)],
    );
}

#[test]
fn cheat_block_timestamp_simple_with_span() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockTimestampChecker", &[]);

    test_env.cheat_block_timestamp(
        contract_address,
        123,
        CheatSpan::TargetCalls(NonZeroUsize::new(2).unwrap()),
    );

    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt::from(DEFAULT_BLOCK_TIMESTAMP)],
    );
}

#[test]
fn cheat_block_timestamp_proxy_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("CheatBlockTimestampCheckerProxy", &contracts_data);
    let contract_address_1 = test_env.deploy_wrapper(&class_hash, &[]);
    let contract_address_2 = test_env.deploy_wrapper(&class_hash, &[]);

    test_env.cheat_block_timestamp(
        contract_address_1,
        123,
        CheatSpan::TargetCalls(NonZeroUsize::new(1).unwrap()),
    );

    let output = test_env.call_contract(
        &contract_address_1,
        "call_proxy",
        &[contract_address_2.into_()],
    );
    assert_success(output, &[123.into(), DEFAULT_BLOCK_TIMESTAMP.into()]);
}

#[test]
fn cheat_block_timestamp_in_constructor_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();

    let class_hash = test_env.declare("ConstructorCheatBlockTimestampChecker", &contracts_data);
    let precalculated_address = test_env.precalculate_address(&class_hash, &[]);

    test_env.cheat_block_timestamp(
        precalculated_address,
        123,
        CheatSpan::TargetCalls(NonZeroUsize::new(2).unwrap()),
    );

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);
    assert_eq!(precalculated_address, contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt::from(DEFAULT_BLOCK_TIMESTAMP)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_stored_block_timestamp", &[]),
        &[Felt::from(123)],
    );
}

#[test]
fn cheat_block_timestamp_no_constructor_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();

    let class_hash = test_env.declare("CheatBlockTimestampChecker", &contracts_data);
    let precalculated_address = test_env.precalculate_address(&class_hash, &[]);

    test_env.cheat_block_timestamp(
        precalculated_address,
        123,
        CheatSpan::TargetCalls(NonZeroUsize::new(1).unwrap()),
    );

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);
    assert_eq!(precalculated_address, contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt::from(DEFAULT_BLOCK_TIMESTAMP)],
    );
}

#[test]
fn cheat_block_timestamp_override_span() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockTimestampChecker", &[]);

    test_env.cheat_block_timestamp(
        contract_address,
        123,
        CheatSpan::TargetCalls(NonZeroUsize::new(2).unwrap()),
    );

    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt::from(123)],
    );

    test_env.cheat_block_timestamp(contract_address, 321, CheatSpan::Indefinite);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt::from(321)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt::from(321)],
    );

    test_env.stop_cheat_block_timestamp(contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        &[Felt::from(DEFAULT_BLOCK_TIMESTAMP)],
    );
}

#[test]
fn cheat_block_timestamp_library_call_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("CheatBlockTimestampChecker", &contracts_data);
    let contract_address = test_env.deploy("CheatBlockTimestampCheckerLibCall", &[]);

    test_env.cheat_block_timestamp(
        contract_address,
        123,
        CheatSpan::TargetCalls(NonZeroUsize::new(1).unwrap()),
    );

    let lib_call_selector = "get_block_timestamp_with_lib_call";

    assert_success(
        test_env.call_contract(&contract_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt::from(DEFAULT_BLOCK_TIMESTAMP)],
    );
}
