use crate::{
    cheatcodes::test_environment::TestEnvironment, common::assertions::assert_success,
    common::get_contracts,
};
use blockifier::state::cached_state::{
    CachedState, GlobalContractCache, GLOBAL_CONTRACT_CACHE_SIZE_FOR_TEST,
};
use cairo_felt::Felt252;
use cheatnet::{
    constants::build_testing_state,
    forking::state::ForkStateReader,
    runtime_extensions::call_to_blockifier_runtime_extension::rpc::CallResult,
    state::{CheatnetState, ExtendedStateReader},
};
use num_traits::Zero;
use starknet_api::{
    block::BlockNumber,
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

impl ReplaceBytecodeTrait for TestEnvironment<'_> {
    fn replace_class_for_contract(
        &mut self,
        contract_address: ContractAddress,
        class_hash: ClassHash,
    ) {
        self.runtime_state
            .cheatnet_state
            .replace_class_for_contract(contract_address, class_hash);
    }
}

#[ignore] // TODO (#1916)
#[test]
fn fork() {
    let cache_dir = TempDir::new().unwrap();
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);
    test_env.cached_state = CachedState::new(
        ExtendedStateReader {
            dict_state_reader: build_testing_state(),
            fork_state_reader: Some(ForkStateReader::new(
                "http://188.34.188.184:6060/rpc/v0_7".parse().unwrap(),
                BlockNumber(957_613),
                cache_dir.path().to_str().unwrap(),
            )),
        },
        GlobalContractCache::new(GLOBAL_CONTRACT_CACHE_SIZE_FOR_TEST),
    );
    let contracts = get_contracts();

    let class_hash = test_env.declare("ReplaceInFork", &contracts);
    let contract =
        contract_address!("0x066ecea8cc2444d33214b9e379be1acef84aa340469b8cd285201e0517c5cb14");

    let output = test_env.call_contract(&contract, "get_admin", &[]);

    assert!(matches!(
        output,
        CallResult::Success { ret_data, .. } if ret_data != [Felt252::zero()],
    ));

    test_env.replace_class_for_contract(contract, class_hash);

    let output = test_env.call_contract(&contract, "get_admin", &[]);

    assert_success(output, &[Felt252::zero()]);
}

#[test]
fn override_entrypoint() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);
    let contracts = get_contracts();

    let class_hash_a = test_env.declare("ReplaceBytecodeA", &contracts);
    let class_hash_b = test_env.declare("ReplaceBytecodeB", &contracts);
    let contract_address = test_env.deploy_wrapper(&class_hash_a, &[]);

    let output = test_env.call_contract(&contract_address, "get_const", &[]);

    assert_success(output, &[Felt252::from(2137)]);

    test_env.replace_class_for_contract(contract_address, class_hash_b);

    let output = test_env.call_contract(&contract_address, "get_const", &[]);

    assert_success(output, &[Felt252::from(420)]);
}

#[test]
fn keep_storage() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);
    let contracts = get_contracts();

    let class_hash_a = test_env.declare("ReplaceBytecodeA", &contracts);
    let class_hash_b = test_env.declare("ReplaceBytecodeB", &contracts);
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
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);
    let contracts = get_contracts();

    let class_hash_a = test_env.declare("ReplaceBytecodeA", &contracts);
    let class_hash_b = test_env.declare("ReplaceBytecodeB", &contracts);
    let contract_address = test_env.deploy_wrapper(&class_hash_a, &[]);

    test_env.replace_class_for_contract(contract_address, class_hash_b);

    test_env.replace_class_for_contract(contract_address, class_hash_a);

    let output = test_env.call_contract(&contract_address, "get_const", &[]);

    assert_success(output, &[Felt252::from(2137)]);
}
