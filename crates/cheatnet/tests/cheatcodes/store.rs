use crate::common::assertions::assert_success;
use crate::common::get_contracts;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::storage::{
    map_entry_address, store, variable_address,
};
use starknet_api::core::ContractAddress;
use starknet_types_core::felt::Felt;

use super::test_environment::TestEnvironment;

trait StoreTrait {
    fn store(&mut self, target: ContractAddress, storage_address: Felt, value: u128);
}

impl StoreTrait for TestEnvironment {
    fn store(&mut self, target: ContractAddress, storage_address: Felt, value: u128) {
        store(
            &mut self.cached_state,
            target,
            storage_address,
            Felt::from(value),
        )
        .unwrap();
    }
}

#[test]
fn store_simple_state() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();

    let class_hash = test_env.declare("HelloStarknet", &contracts_data);
    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);

    test_env.store(contract_address, variable_address("balance"), 666);

    assert_success(
        test_env.call_contract(&contract_address, "get_balance", &[]),
        &[Felt::from(666)],
    );
}

#[test]
fn store_state_map_simple_value() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();

    let class_hash = test_env.declare("MapSimpleValueSimpleKey", &contracts_data);

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);

    let map_key = Felt::from(420);
    let inserted_value = 69;

    let entry_address = map_entry_address("values", &[map_key]);
    test_env.store(contract_address, entry_address, inserted_value);

    assert_success(
        test_env.call_contract(&contract_address, "read", &[map_key]),
        &[inserted_value.into()],
    );
}
