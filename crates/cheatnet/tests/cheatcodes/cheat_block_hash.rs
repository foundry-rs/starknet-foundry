use super::test_environment::TestEnvironment;
use crate::common::assertions::assert_success;
use cairo_vm::Felt252;
use runtime::starknet::context::DEFAULT_BLOCK_NUMBER;
use starknet_types_core::felt::Felt;

trait CheatBlockHashTrait {
    fn cheat_block_hash(&mut self, block_number: u64, block_hash: Felt);
}

impl CheatBlockHashTrait for TestEnvironment {
    fn cheat_block_hash(&mut self, block_number: u64, block_hash: Felt) {
        self.cheatnet_state.set_block_hash(block_number, block_hash);
    }
}

#[test]
fn cheat_block_hash_simple() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockHashChecker", &[]);

    test_env.cheat_block_hash(1990, Felt252::from(123));

    let output = test_env.call_contract(&contract_address, "get_block_hash", &[]);
    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn cheat_block_hash_with_other_syscall() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockHashChecker", &[]);

    let output = test_env.call_contract(&contract_address, "get_block_hash_and_number", &[]);

    assert_success(
        output,
        &[Felt252::from(0), Felt252::from(DEFAULT_BLOCK_NUMBER - 10)],
    );

    test_env.cheat_block_hash(1990, Felt252::from(123));

    let output = test_env.call_contract(&contract_address, "get_block_hash", &[]);
    assert_success(output, &[Felt252::from(123)]);
}
