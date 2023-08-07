use crate::{cheatcodes::EnhancedHintError, CheatedState};
use starknet_api::core::{ContractAddress, EntryPointSelector};
use starknet_api::hash::StarkFelt;
use std::collections::HashMap;

impl CheatedState {
    pub fn start_mock_call(
        &mut self,
        contract_address: ContractAddress,
        function_name: EntryPointSelector,
        ret_data: Vec<StarkFelt>,
    ) -> Result<(), EnhancedHintError> {
        let contract_mocked_functions = self.mocked_functions
            .entry(contract_address)
            .or_insert_with(HashMap::new);

        contract_mocked_functions.insert(function_name, ret_data);

        Ok(())
    }

    pub fn stop_mock_call(
        &mut self,
        contract_address: ContractAddress,
        function_name: EntryPointSelector,
    ) -> Result<(), EnhancedHintError> {
        if let std::collections::hash_map::Entry::Occupied(mut e) =
            self.mocked_functions.entry(contract_address)
        {
            let contract_mocked_functions = e.get_mut();
            contract_mocked_functions.remove(&function_name);
        }

        Ok(())
    }
}
