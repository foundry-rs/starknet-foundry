use crate::{cheatcodes::EnhancedHintError, CheatedState};
use starknet_api::core::{ContractAddress, EntryPointSelector};
use starknet_api::hash::StarkFelt;
use std::collections::HashMap;

impl CheatedState {
    pub fn start_mock_call(
        &mut self,
        contract_address: ContractAddress,
        fn_name: EntryPointSelector,
        ret_data: Vec<StarkFelt>,
    ) -> Result<(), EnhancedHintError> {
        if self.mocked_functions.contains_key(&contract_address) {
            let contract_mocked_fns = self.mocked_functions.get_mut(&contract_address).unwrap();
            contract_mocked_fns.insert(fn_name, ret_data);
        } else {
            self.mocked_functions
                .insert(contract_address, HashMap::from([(fn_name, ret_data)]));
        }

        Ok(())
    }
}
