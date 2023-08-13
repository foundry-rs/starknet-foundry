use crate::CheatnetState;
use starknet_api::core::{ContractAddress, EntryPointSelector};
use starknet_api::hash::StarkFelt;
use std::collections::HashMap;

impl CheatnetState {
    pub fn start_mock_call(
        &mut self,
        contract_address: ContractAddress,
        function_name: EntryPointSelector,
        ret_data: Vec<StarkFelt>,
    ) {
        let contract_mocked_functions = self
            .cheatcode_state
            .mocked_functions
            .entry(contract_address)
            .or_insert_with(HashMap::new);

        contract_mocked_functions.insert(function_name, ret_data);
    }

    pub fn stop_mock_call(
        &mut self,
        contract_address: ContractAddress,
        function_name: EntryPointSelector,
    ) {
        if let std::collections::hash_map::Entry::Occupied(mut e) = self
            .cheatcode_state
            .mocked_functions
            .entry(contract_address)
        {
            let contract_mocked_functions = e.get_mut();
            contract_mocked_functions.remove(&function_name);
        }
    }
}
