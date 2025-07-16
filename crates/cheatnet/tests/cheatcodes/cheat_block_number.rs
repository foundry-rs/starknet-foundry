use crate::{common::assertions::assert_success, common::get_contracts};
use cheatnet::state::CheatSpan;
use conversions::IntoConv;
use runtime::starknet::context::DEFAULT_BLOCK_NUMBER;
use starknet_api::core::ContractAddress;
use starknet_types_core::felt::Felt;
use std::num::NonZeroUsize;

use super::test_environment::TestEnvironment;

trait CheatBlockNumberTrait {
    fn cheat_block_number(
        &mut self,
        contract_address: ContractAddress,
        block_number: u64,
        span: CheatSpan,
    );
    fn start_cheat_block_number(&mut self, contract_address: ContractAddress, block_number: u64);
    fn stop_cheat_block_number(&mut self, contract_address: ContractAddress);
}

impl CheatBlockNumberTrait for TestEnvironment {
    fn cheat_block_number(
        &mut self,
        contract_address: ContractAddress,
        block_number: u64,
        span: CheatSpan,
    ) {
        self.cheatnet_state
            .cheat_block_number(contract_address, block_number, span);
    }

    fn start_cheat_block_number(&mut self, contract_address: ContractAddress, block_number: u64) {
        self.cheatnet_state
            .start_cheat_block_number(contract_address, block_number);
    }

    fn stop_cheat_block_number(&mut self, contract_address: ContractAddress) {
        self.cheatnet_state
            .stop_cheat_block_number(contract_address);
    }
}

#[test]
fn cheat_block_number_simple() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockNumberChecker", &[]);

    test_env.start_cheat_block_number(contract_address, 123);

    let output = test_env.call_contract(&contract_address, "get_block_number", &[]);
    assert_success(output, &[Felt::from(123)]);
}

#[test]
fn cheat_block_number_with_other_syscall() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockNumberChecker", &[]);

    test_env.start_cheat_block_number(contract_address, 123);

    let output = test_env.call_contract(&contract_address, "get_block_number_and_emit_event", &[]);
    assert_success(output, &[Felt::from(123)]);
}

#[test]
fn cheat_block_number_in_constructor() {
    let mut test_env = TestEnvironment::new();
    let contracts_data = get_contracts();

    let class_hash = test_env.declare("ConstructorCheatBlockNumberChecker", &contracts_data);
    let precalculated_address = test_env.precalculate_address(&class_hash, &[]);

    test_env.start_cheat_block_number(precalculated_address, 123);

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);
    assert_eq!(precalculated_address, contract_address);

    let output = test_env.call_contract(&contract_address, "get_stored_block_number", &[]);
    assert_success(output, &[Felt::from(123)]);
}

#[test]
fn cheat_block_number_stop() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockNumberChecker", &[]);

    test_env.start_cheat_block_number(contract_address, 123);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt::from(123)],
    );

    test_env.stop_cheat_block_number(contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt::from(DEFAULT_BLOCK_NUMBER)],
    );
}

#[test]
fn cheat_block_number_double() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockNumberChecker", &[]);

    test_env.start_cheat_block_number(contract_address, 123);
    test_env.start_cheat_block_number(contract_address, 123);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt::from(123)],
    );

    test_env.stop_cheat_block_number(contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt::from(DEFAULT_BLOCK_NUMBER)],
    );
}

#[test]
fn cheat_block_number_proxy() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockNumberChecker", &[]);
    let proxy_address = test_env.deploy("CheatBlockNumberCheckerProxy", &[]);

    let proxy_selector = "get_cheated_block_number";

    test_env.start_cheat_block_number(contract_address, 123);

    assert_success(
        test_env.call_contract(&proxy_address, proxy_selector, &[contract_address.into_()]),
        &[Felt::from(123)],
    );

    test_env.stop_cheat_block_number(contract_address);

    assert_success(
        test_env.call_contract(&proxy_address, proxy_selector, &[contract_address.into_()]),
        &[Felt::from(DEFAULT_BLOCK_NUMBER)],
    );
}

#[test]
fn cheat_block_number_library_call() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("CheatBlockNumberChecker", &contracts_data);

    let lib_call_address = test_env.deploy("CheatBlockNumberCheckerLibCall", &[]);
    let lib_call_selector = "get_block_number_with_lib_call";

    test_env.start_cheat_block_number(lib_call_address, 123);

    assert_success(
        test_env.call_contract(&lib_call_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt::from(123)],
    );
    test_env.stop_cheat_block_number(lib_call_address);

    assert_success(
        test_env.call_contract(&lib_call_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt::from(DEFAULT_BLOCK_NUMBER)],
    );
}

#[test]
fn cheat_block_number_all_simple() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockNumberChecker", &[]);

    test_env.cheatnet_state.start_cheat_block_number_global(123);

    let output = test_env.call_contract(&contract_address, "get_block_number", &[]);
    assert_success(output, &[Felt::from(123)]);
}

#[test]
fn cheat_block_number_all_then_one() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockNumberChecker", &[]);

    test_env.cheatnet_state.start_cheat_block_number_global(321);
    test_env.start_cheat_block_number(contract_address, 123);

    let output = test_env.call_contract(&contract_address, "get_block_number", &[]);
    assert_success(output, &[Felt::from(123)]);
}

#[test]
fn cheat_block_number_one_then_all() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockNumberChecker", &[]);

    test_env.start_cheat_block_number(contract_address, 123);
    test_env.cheatnet_state.start_cheat_block_number_global(321);

    let output = test_env.call_contract(&contract_address, "get_block_number", &[]);
    assert_success(output, &[Felt::from(321)]);
}

#[test]
fn cheat_block_number_all_stop() {
    let mut test_env = TestEnvironment::new();

    let cheat_block_number_checker = test_env.declare("CheatBlockNumberChecker", &get_contracts());
    let contract_address = test_env.deploy_wrapper(&cheat_block_number_checker, &[]);

    test_env.cheatnet_state.start_cheat_block_number_global(123);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt::from(123)],
    );

    test_env.cheatnet_state.stop_cheat_block_number_global();

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt::from(DEFAULT_BLOCK_NUMBER)],
    );

    let contract_address = test_env.deploy_wrapper(&cheat_block_number_checker, &[]);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt::from(DEFAULT_BLOCK_NUMBER)],
    );
}

#[test]
fn cheat_block_number_multiple() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("CheatBlockNumberChecker", &contracts_data);

    let contract_address1 = test_env.deploy_wrapper(&class_hash, &[]);
    let contract_address2 = test_env.deploy_wrapper(&class_hash, &[]);

    test_env
        .cheatnet_state
        .start_cheat_block_number(contract_address1, 123);
    test_env
        .cheatnet_state
        .start_cheat_block_number(contract_address2, 123);

    assert_success(
        test_env.call_contract(&contract_address1, "get_block_number", &[]),
        &[Felt::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address2, "get_block_number", &[]),
        &[Felt::from(123)],
    );

    test_env
        .cheatnet_state
        .stop_cheat_block_number(contract_address1);
    test_env
        .cheatnet_state
        .stop_cheat_block_number(contract_address2);

    assert_success(
        test_env.call_contract(&contract_address1, "get_block_number", &[]),
        &[Felt::from(DEFAULT_BLOCK_NUMBER)],
    );
    assert_success(
        test_env.call_contract(&contract_address2, "get_block_number", &[]),
        &[Felt::from(DEFAULT_BLOCK_NUMBER)],
    );
}

#[test]
fn cheat_block_number_simple_with_span() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockNumberChecker", &[]);

    test_env.cheat_block_number(
        contract_address,
        123,
        CheatSpan::TargetCalls(NonZeroUsize::new(2).unwrap()),
    );

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt::from(DEFAULT_BLOCK_NUMBER)],
    );
}

#[test]
fn cheat_block_number_proxy_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("CheatBlockNumberCheckerProxy", &contracts_data);
    let contract_address_1 = test_env.deploy_wrapper(&class_hash, &[]);
    let contract_address_2 = test_env.deploy_wrapper(&class_hash, &[]);

    test_env.cheat_block_number(
        contract_address_1,
        123,
        CheatSpan::TargetCalls(NonZeroUsize::new(1).unwrap()),
    );

    let output = test_env.call_contract(
        &contract_address_1,
        "call_proxy",
        &[contract_address_2.into_()],
    );
    assert_success(output, &[123.into(), DEFAULT_BLOCK_NUMBER.into()]);
}

#[test]
fn cheat_block_number_in_constructor_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();

    let class_hash = test_env.declare("ConstructorCheatBlockNumberChecker", &contracts_data);
    let precalculated_address = test_env.precalculate_address(&class_hash, &[]);

    test_env.cheat_block_number(
        precalculated_address,
        123,
        CheatSpan::TargetCalls(NonZeroUsize::new(2).unwrap()),
    );

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);
    assert_eq!(precalculated_address, contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt::from(DEFAULT_BLOCK_NUMBER)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_stored_block_number", &[]),
        &[Felt::from(123)],
    );
}

#[test]
fn cheat_block_number_no_constructor_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();

    let class_hash = test_env.declare("CheatBlockNumberChecker", &contracts_data);
    let precalculated_address = test_env.precalculate_address(&class_hash, &[]);

    test_env.cheat_block_number(
        precalculated_address,
        123,
        CheatSpan::TargetCalls(NonZeroUsize::new(1).unwrap()),
    );

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);
    assert_eq!(precalculated_address, contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt::from(DEFAULT_BLOCK_NUMBER)],
    );
}

#[test]
fn cheat_block_number_override_span() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockNumberChecker", &[]);

    test_env.cheat_block_number(
        contract_address,
        123,
        CheatSpan::TargetCalls(NonZeroUsize::new(2).unwrap()),
    );

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt::from(123)],
    );

    test_env.cheat_block_number(contract_address, 321, CheatSpan::Indefinite);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt::from(321)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt::from(321)],
    );

    test_env.stop_cheat_block_number(contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt::from(DEFAULT_BLOCK_NUMBER)],
    );
}

#[test]
fn cheat_block_number_library_call_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("CheatBlockNumberChecker", &contracts_data);
    let contract_address = test_env.deploy("CheatBlockNumberCheckerLibCall", &[]);

    test_env.cheat_block_number(
        contract_address,
        123,
        CheatSpan::TargetCalls(NonZeroUsize::new(1).unwrap()),
    );

    let lib_call_selector = "get_block_number_with_lib_call";

    assert_success(
        test_env.call_contract(&contract_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt::from(DEFAULT_BLOCK_NUMBER)],
    );
}
