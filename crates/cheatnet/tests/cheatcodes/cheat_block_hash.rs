use crate::{common::assertions::assert_success, common::get_contracts};
use cairo_vm::Felt252;
use cheatnet::state::CheatSpan;
use conversions::IntoConv;
use starknet_api::core::ContractAddress;

use super::test_environment::TestEnvironment;

// use runtime::starknet::context::DEFAULT_BLOCK_HASH;

const DEFAULT_BLOCK_HASH: u64 = 0;

trait CheatBlockHashTrait {
    fn cheat_block_hash(
        &mut self,
        contract_address: ContractAddress,
        block_hash: Felt252,
        span: CheatSpan,
    );
    fn start_cheat_block_hash(&mut self, contract_address: ContractAddress, block_hash: Felt252);
    fn stop_cheat_block_hash(&mut self, contract_address: ContractAddress);
}

impl CheatBlockHashTrait for TestEnvironment {
    fn cheat_block_hash(
        &mut self,
        contract_address: ContractAddress,
        block_hash: Felt252,
        span: CheatSpan,
    ) {
        self.cheatnet_state
            .cheat_block_hash(contract_address, block_hash, span);
    }

    fn start_cheat_block_hash(&mut self, contract_address: ContractAddress, block_hash: Felt252) {
        self.cheatnet_state
            .start_cheat_block_hash(contract_address, block_hash);
    }

    fn stop_cheat_block_hash(&mut self, contract_address: ContractAddress) {
        self.cheatnet_state.stop_cheat_block_hash(contract_address);
    }
}

#[test]
fn cheat_block_hash_simple() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockHashChecker", &[]);

    test_env.start_cheat_block_hash(contract_address, Felt252::from(123));

    let output = test_env.call_contract(&contract_address, "get_block_hash", &[]);
    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn cheat_block_hash_with_other_syscall() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockHashChecker", &[]);

    test_env.start_cheat_block_hash(contract_address, Felt252::from(123));
    let selector = "get_block_hash_and_emit_event";

    let output = test_env.call_contract(&contract_address, selector, &[]);
    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn cheat_block_hash_in_constructor() {
    let mut test_env = TestEnvironment::new();
    let contracts_data = get_contracts();

    let class_hash = test_env.declare("ConstructorCheatBlockHashChecker", &contracts_data);
    let precalculated_address = test_env.precalculate_address(&class_hash, &[]);

    test_env.start_cheat_block_hash(precalculated_address, Felt252::from(123));

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);

    assert_eq!(precalculated_address, contract_address);

    let output = test_env.call_contract(&contract_address, "get_stored_block_hash", &[]);
    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn cheat_block_hash_stop() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockHashChecker", &[]);

    test_env.start_cheat_block_hash(contract_address, Felt252::from(123));

    assert_success(
        test_env.call_contract(&contract_address, "get_block_hash", &[]),
        &[Felt252::from(123)],
    );

    test_env.stop_cheat_block_hash(contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_hash", &[]),
        &[Felt252::from(DEFAULT_BLOCK_HASH)],
    );
}

#[test]
fn cheat_block_hash_double() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockHashChecker", &[]);

    test_env.start_cheat_block_hash(contract_address, Felt252::from(123));
    test_env.start_cheat_block_hash(contract_address, Felt252::from(123));

    assert_success(
        test_env.call_contract(&contract_address, "get_block_hash", &[]),
        &[Felt252::from(123)],
    );

    test_env.stop_cheat_block_hash(contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_hash", &[]),
        &[Felt252::from(DEFAULT_BLOCK_HASH)],
    );
}

#[test]
fn cheat_block_hash_proxy() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockHashChecker", &[]);
    let proxy_address = test_env.deploy("CheatBlockHashCheckerProxy", &[]);
    let proxy_selector = "get_cheated_block_hash";

    test_env.start_cheat_block_hash(contract_address, Felt252::from(123));

    assert_success(
        test_env.call_contract(&proxy_address, proxy_selector, &[contract_address.into_()]),
        &[Felt252::from(123)],
    );

    test_env.stop_cheat_block_hash(contract_address);

    assert_success(
        test_env.call_contract(&proxy_address, proxy_selector, &[contract_address.into_()]),
        &[Felt252::from(DEFAULT_BLOCK_HASH)],
    );
}

#[test]
fn cheat_block_hash_library_call() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("CheatBlockHashChecker", &contracts_data);

    let lib_call_address = test_env.deploy("CheatBlockHashCheckerLibCall", &[]);
    let lib_call_selector = "get_block_hash_with_lib_call";

    test_env.start_cheat_block_hash(lib_call_address, Felt252::from(123));

    assert_success(
        test_env.call_contract(&lib_call_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt252::from(123)],
    );
    test_env.stop_cheat_block_hash(lib_call_address);

    assert_success(
        test_env.call_contract(&lib_call_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt252::from(DEFAULT_BLOCK_HASH)],
    );
}

#[test]
fn cheat_block_hash_all_simple() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockHashChecker", &[]);

    test_env
        .cheatnet_state
        .start_cheat_block_hash_global(Felt252::from(123));

    let output = test_env.call_contract(&contract_address, "get_block_hash", &[]);
    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn cheat_block_hash_all_then_one() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockHashChecker", &[]);

    test_env
        .cheatnet_state
        .start_cheat_block_hash_global(Felt252::from(321));

    test_env.start_cheat_block_hash(contract_address, Felt252::from(123));

    let output = test_env.call_contract(&contract_address, "get_block_hash", &[]);
    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn cheat_block_hash_one_then_all() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockHashChecker", &[]);

    test_env.start_cheat_block_hash(contract_address, Felt252::from(123));

    test_env
        .cheatnet_state
        .start_cheat_block_hash_global(Felt252::from(321));

    let output = test_env.call_contract(&contract_address, "get_block_hash", &[]);
    assert_success(output, &[Felt252::from(321)]);
}

#[test]
fn cheat_block_hash_all_stop() {
    let mut test_env = TestEnvironment::new();

    let cheat_block_hash_checker = test_env.declare("CheatBlockHashChecker", &get_contracts());

    let contract_address = test_env.deploy_wrapper(&cheat_block_hash_checker, &[]);

    test_env
        .cheatnet_state
        .start_cheat_block_hash_global(Felt252::from(123));

    assert_success(
        test_env.call_contract(&contract_address, "get_block_hash", &[]),
        &[Felt252::from(123)],
    );

    test_env.cheatnet_state.stop_cheat_block_hash_global();

    assert_success(
        test_env.call_contract(&contract_address, "get_block_hash", &[]),
        &[Felt252::from(DEFAULT_BLOCK_HASH)],
    );

    let contract_address = test_env.deploy_wrapper(&cheat_block_hash_checker, &[]);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_hash", &[]),
        &[Felt252::from(DEFAULT_BLOCK_HASH)],
    );
}

#[test]
fn cheat_block_hash_multiple() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("CheatBlockHashChecker", &contracts_data);

    let contract_address1 = test_env.deploy_wrapper(&class_hash, &[]);
    let contract_address2 = test_env.deploy_wrapper(&class_hash, &[]);

    test_env
        .cheatnet_state
        .start_cheat_block_hash(contract_address1, Felt252::from(123));
    test_env
        .cheatnet_state
        .start_cheat_block_hash(contract_address2, Felt252::from(123));

    assert_success(
        test_env.call_contract(&contract_address1, "get_block_hash", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address2, "get_block_hash", &[]),
        &[Felt252::from(123)],
    );

    test_env
        .cheatnet_state
        .stop_cheat_block_hash(contract_address1);
    test_env
        .cheatnet_state
        .stop_cheat_block_hash(contract_address2);

    assert_success(
        test_env.call_contract(&contract_address1, "get_block_hash", &[]),
        &[Felt252::from(DEFAULT_BLOCK_HASH)],
    );
    assert_success(
        test_env.call_contract(&contract_address2, "get_block_hash", &[]),
        &[Felt252::from(DEFAULT_BLOCK_HASH)],
    );
}

#[test]
fn cheat_block_hash_simple_with_span() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockHashChecker", &[]);

    test_env.cheat_block_hash(
        contract_address,
        Felt252::from(123),
        CheatSpan::TargetCalls(2),
    );

    assert_success(
        test_env.call_contract(&contract_address, "get_block_hash", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_hash", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_hash", &[]),
        &[Felt252::from(DEFAULT_BLOCK_HASH)],
    );
}

#[test]
fn cheat_block_hash_proxy_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("CheatBlockHashCheckerProxy", &contracts_data);
    let contract_address_1 = test_env.deploy_wrapper(&class_hash, &[]);
    let contract_address_2 = test_env.deploy_wrapper(&class_hash, &[]);

    test_env.cheat_block_hash(
        contract_address_1,
        Felt252::from(123),
        CheatSpan::TargetCalls(1),
    );

    let output = test_env.call_contract(
        &contract_address_1,
        "call_proxy",
        &[contract_address_2.into_()],
    );
    assert_success(output, &[123.into(), DEFAULT_BLOCK_HASH.into()]);
}

#[test]
fn cheat_block_hash_in_constructor_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();

    let class_hash = test_env.declare("ConstructorCheatBlockHashChecker", &contracts_data);
    let precalculated_address = test_env.precalculate_address(&class_hash, &[]);

    test_env.cheat_block_hash(
        precalculated_address,
        Felt252::from(123),
        CheatSpan::TargetCalls(2),
    );

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);
    assert_eq!(precalculated_address, contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_hash", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_hash", &[]),
        &[Felt252::from(DEFAULT_BLOCK_HASH)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_stored_block_hash", &[]),
        &[Felt252::from(123)],
    );
}

#[test]
fn cheat_block_hash_no_constructor_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();

    let class_hash = test_env.declare("CheatBlockHashChecker", &contracts_data);
    let precalculated_address = test_env.precalculate_address(&class_hash, &[]);

    test_env.cheat_block_hash(
        precalculated_address,
        Felt252::from(123),
        CheatSpan::TargetCalls(1),
    );

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);
    assert_eq!(precalculated_address, contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_hash", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_hash", &[]),
        &[Felt252::from(DEFAULT_BLOCK_HASH)],
    );
}

#[test]
fn cheat_block_hash_override_span() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockHashChecker", &[]);

    test_env.cheat_block_hash(
        contract_address,
        Felt252::from(123),
        CheatSpan::TargetCalls(2),
    );

    assert_success(
        test_env.call_contract(&contract_address, "get_block_hash", &[]),
        &[Felt252::from(123)],
    );

    test_env.cheat_block_hash(contract_address, Felt252::from(321), CheatSpan::Indefinite);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_hash", &[]),
        &[Felt252::from(321)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_hash", &[]),
        &[Felt252::from(321)],
    );

    test_env.stop_cheat_block_hash(contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_hash", &[]),
        &[Felt252::from(DEFAULT_BLOCK_HASH)],
    );
}

#[test]
fn cheat_block_hash_library_call_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("CheatBlockHashChecker", &contracts_data);
    let contract_address = test_env.deploy("CheatBlockHashCheckerLibCall", &[]);

    test_env.cheat_block_hash(
        contract_address,
        Felt252::from(123),
        CheatSpan::TargetCalls(1),
    );

    let lib_call_selector = "get_block_hash_with_lib_call";

    assert_success(
        test_env.call_contract(&contract_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt252::from(DEFAULT_BLOCK_HASH)],
    );
}
