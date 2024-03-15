use crate::cheatcodes::{map_entry_address, variable_address};
use crate::common::get_contracts;
use cairo_felt::Felt252;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::storage::load;
use cheatnet::state::CheatnetState;
use starknet_api::core::ContractAddress;

use super::test_environment::TestEnvironment;

trait LoadTrait {
    fn load(&mut self, target: ContractAddress, storage_address: &Felt252) -> Felt252;
}

impl<'a> LoadTrait for TestEnvironment<'a> {
    fn load(&mut self, target: ContractAddress, storage_address: &Felt252) -> Felt252 {
        load(&mut self.cached_state, target, storage_address).unwrap()
    }
}

#[test]
fn load_simple_state() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);
    let contracts = get_contracts();

    let class_hash = test_env.declare("HelloStarknet", &contracts);
    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);

    test_env.call_contract(&contract_address, "increase_balance", &[Felt252::from(420)]);

    let balance_value = test_env.load(contract_address, &variable_address("balance"));

    assert_eq!(
        balance_value,
        Felt252::from(420),
        "Wrong data value was returned: {balance_value}"
    );
}

#[test]
fn load_state_map_simple_value() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);
    let contracts = get_contracts();

    let class_hash = test_env.declare("MapSimpleValueSimpleKey", &contracts);
    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);

    let map_key = Felt252::from(420);
    let inserted_value = Felt252::from(69);
    test_env.call_contract(
        &contract_address,
        "insert",
        &[map_key.clone(), inserted_value.clone()],
    );

    let var_address = map_entry_address("values", &[map_key]);
    let map_value = test_env.load(contract_address, &var_address);

    assert_eq!(
        map_value, inserted_value,
        "Wrong data value was returned: {map_value}"
    );
}
