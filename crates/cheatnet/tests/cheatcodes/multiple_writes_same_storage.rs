use crate::cheatcodes::variable_address;
use crate::common::assertions::assert_success;
use crate::common::get_contracts;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::storage::{load, store};
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
    let contract_address_write = test_env.deploy_wrapper(&hello_class_hash, &[]);
    let contract_address_read = test_env.deploy_wrapper(&hello_class_hash, &[]);
    assert_ne!(contract_address_read, contract_address_write);

    test_env.call_contract(
        &contract_address_write,
        "increase_balance",
        &[Felt::from(420)],
    );
    let write_balance_value = test_env.call_contract(&contract_address_write, "get_balance", &[]);
    assert_success(write_balance_value, &[Felt::from(420)]);

    let read_balance_value = test_env.call_contract(&contract_address_read, "get_balance", &[]);
    assert_success(read_balance_value, &[Felt::from(0)]);

    test_env.call_contract(
        &contract_address_read,
        "increase_balance",
        &[Felt::from(42)],
    );

    let read_balance_value = test_env.call_contract(&contract_address_read, "get_balance", &[]);
    assert_success(read_balance_value, &[Felt::from(42)]);

    let write_balance_value = test_env.call_contract(&contract_address_write, "get_balance", &[]);
    assert_success(write_balance_value, &[Felt::from(420)]);
}

#[test]
fn same_storage_access_store() {
    let mut test_env = TestEnvironment::new();
    let contracts_data = get_contracts();
    let hello_class_hash = test_env.declare("HelloStarknet", &contracts_data);
    let contract_address_write = test_env.deploy_wrapper(&hello_class_hash, &[]);
    let contract_address_read = test_env.deploy_wrapper(&hello_class_hash, &[]);
    assert_ne!(contract_address_read, contract_address_write);

    test_env.store(contract_address_write, variable_address("balance"), 450);
    let write_balance_value = test_env.load(contract_address_write, variable_address("balance"));
    assert_eq!(write_balance_value, Felt::from(450));

    let read_balance_value = test_env.load(contract_address_read, variable_address("balance"));
    assert_eq!(read_balance_value, Felt::from(0));

    test_env.store(contract_address_read, variable_address("balance"), 42);
    let read_balance_value = test_env.load(contract_address_read, variable_address("balance"));
    assert_eq!(read_balance_value, Felt::from(42));

    let write_balance_value = test_env.load(contract_address_write, variable_address("balance"));
    assert_eq!(write_balance_value, Felt::from(450));
}
