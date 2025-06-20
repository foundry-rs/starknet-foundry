use crate::common::assertions::assert_success;
use crate::common::get_contracts;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::storage::{
    load, store, variable_address,
};
use starknet_api::core::ContractAddress;
use starknet_types_core::felt::Felt;

use super::test_environment::TestEnvironment;

trait StoreTrait {
    fn store(&mut self, target: ContractAddress, storage_address: Felt, value: u128);
    fn load(&mut self, target: ContractAddress, storage_address: Felt) -> Felt;
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

    fn load(&mut self, target: ContractAddress, storage_address: Felt) -> Felt {
        load(&mut self.cached_state, target, storage_address).unwrap()
    }
}

#[test]
fn same_storage_access_call_contract() {
    let mut test_env = TestEnvironment::new();
    let contracts_data = get_contracts();
    let hello_class_hash = test_env.declare("HelloStarknet", &contracts_data);
    let contract_address_a = test_env.deploy_wrapper(&hello_class_hash, &[]);
    let contract_address_b = test_env.deploy_wrapper(&hello_class_hash, &[]);
    assert_ne!(contract_address_b, contract_address_a);

    test_env.call_contract(&contract_address_a, "increase_balance", &[Felt::from(420)]);
    let balance_value_a = test_env.call_contract(&contract_address_a, "get_balance", &[]);
    assert_success(balance_value_a, &[Felt::from(420)]);

    let balance_value_b = test_env.call_contract(&contract_address_b, "get_balance", &[]);
    assert_success(balance_value_b, &[Felt::from(0)]);

    test_env.call_contract(&contract_address_b, "increase_balance", &[Felt::from(42)]);

    let balance_value_b = test_env.call_contract(&contract_address_b, "get_balance", &[]);
    assert_success(balance_value_b, &[Felt::from(42)]);

    let balance_value_a = test_env.call_contract(&contract_address_a, "get_balance", &[]);
    assert_success(balance_value_a, &[Felt::from(420)]);
}

#[test]
fn same_storage_access_store() {
    let mut test_env = TestEnvironment::new();
    let contracts_data = get_contracts();
    let hello_class_hash = test_env.declare("HelloStarknet", &contracts_data);
    let contract_address_a = test_env.deploy_wrapper(&hello_class_hash, &[]);
    let contract_address_b = test_env.deploy_wrapper(&hello_class_hash, &[]);
    assert_ne!(contract_address_b, contract_address_a);

    test_env.store(contract_address_a, variable_address("balance"), 450);
    let balance_value_a = test_env.load(contract_address_a, variable_address("balance"));
    assert_eq!(balance_value_a, Felt::from(450));

    let balance_value_b = test_env.load(contract_address_b, variable_address("balance"));
    assert_eq!(balance_value_b, Felt::from(0));

    test_env.store(contract_address_b, variable_address("balance"), 42);
    let balance_value_b = test_env.load(contract_address_b, variable_address("balance"));
    assert_eq!(balance_value_b, Felt::from(42));

    let balance_value_a = test_env.load(contract_address_a, variable_address("balance"));
    assert_eq!(balance_value_a, Felt::from(450));
}
