use crate::{
    cheatcodes::test_environment::TestEnvironment,
    common::{assertions::assert_success, get_contracts, state::create_fork_cached_state_at},
};
use cairo_felt::Felt252;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::CallResult;
use num_traits::Zero;
use starknet_api::{
    contract_address,
    core::{ClassHash, ContractAddress, PatriciaKey},
    hash::StarkHash,
    patricia_key,
};
use tempfile::TempDir;

trait ReplaceBytecodeTrait {
    fn replace_class_for_contract(
        &mut self,
        contract_address: ContractAddress,
        class_hash: ClassHash,
    );
}

impl ReplaceBytecodeTrait for TestEnvironment {
    fn replace_class_for_contract(
        &mut self,
        contract_address: ContractAddress,
        class_hash: ClassHash,
    ) {
        self.cheatnet_state
            .replace_class_for_contract(contract_address, class_hash);
    }
}

#[test]
fn fork() {
    let cache_dir = TempDir::new().unwrap();
    let mut test_env = TestEnvironment::new();
    test_env.cached_state = create_fork_cached_state_at(53_300, cache_dir.path().to_str().unwrap());
    let contracts_data = get_contracts();

    let class_hash = test_env.declare("ReplaceInFork", &contracts_data);
    let contract =
        contract_address!("0x06fdb5ef99e9def44484a3f8540bc42333e321e9b24a397d6bc0c8860bb7da8f");

    let output = test_env.call_contract(&contract, "get_owner", &[]);

    assert!(matches!(
        output,
        CallResult::Success { ret_data, .. } if ret_data != [Felt252::zero()],
    ));

    test_env.replace_class_for_contract(contract, class_hash);

    let output = test_env.call_contract(&contract, "get_owner", &[]);

    assert_success(output, &[Felt252::zero()]);
}

#[test]
fn override_entrypoint() {
    let mut test_env = TestEnvironment::new();
    let contracts_data = get_contracts();

    let class_hash_a = test_env.declare("ReplaceBytecodeA", &contracts_data);
    let class_hash_b = test_env.declare("ReplaceBytecodeB", &contracts_data);
    let contract_address = test_env.deploy_wrapper(&class_hash_a, &[]);

    let output = test_env.call_contract(&contract_address, "get_const", &[]);

    assert_success(output, &[Felt252::from(2137)]);

    test_env.replace_class_for_contract(contract_address, class_hash_b);

    let output = test_env.call_contract(&contract_address, "get_const", &[]);

    assert_success(output, &[Felt252::from(420)]);
}

#[test]
fn keep_storage() {
    let mut test_env = TestEnvironment::new();
    let contracts_data = get_contracts();

    let class_hash_a = test_env.declare("ReplaceBytecodeA", &contracts_data);
    let class_hash_b = test_env.declare("ReplaceBytecodeB", &contracts_data);
    let contract_address = test_env.deploy_wrapper(&class_hash_a, &[]);

    let output = test_env.call_contract(&contract_address, "set", &[456.into()]);

    assert_success(output, &[]);

    let output = test_env.call_contract(&contract_address, "get", &[]);

    assert_success(output, &[Felt252::from(456)]);

    test_env.replace_class_for_contract(contract_address, class_hash_b);

    let output = test_env.call_contract(&contract_address, "get", &[]);

    assert_success(output, &[Felt252::from(556)]);
}

#[test]
fn allow_setting_original_class() {
    let mut test_env = TestEnvironment::new();
    let contracts_data = get_contracts();

    let class_hash_a = test_env.declare("ReplaceBytecodeA", &contracts_data);
    let class_hash_b = test_env.declare("ReplaceBytecodeB", &contracts_data);
    let contract_address = test_env.deploy_wrapper(&class_hash_a, &[]);

    test_env.replace_class_for_contract(contract_address, class_hash_b);

    test_env.replace_class_for_contract(contract_address, class_hash_a);

    let output = test_env.call_contract(&contract_address, "get_const", &[]);

    assert_success(output, &[Felt252::from(2137)]);
}
