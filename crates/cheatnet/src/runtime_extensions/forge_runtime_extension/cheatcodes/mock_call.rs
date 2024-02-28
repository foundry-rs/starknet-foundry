use crate::CheatnetState;
use blockifier::execution::execution_utils::felt_to_stark_felt;
use cairo_felt::Felt252;
use conversions::IntoConv;
use starknet_api::core::{ContractAddress, EntryPointSelector};
use starknet_api::hash::StarkFelt;

impl CheatnetState {
    pub fn start_mock_call(
        &mut self,
        contract_address: ContractAddress,
        function_selector: Felt252,
        ret_data: &[Felt252],
    ) {
        let ret_data: Vec<StarkFelt> = ret_data.iter().map(felt_to_stark_felt).collect();

        let contract_mocked_functions = self.mocked_functions.entry(contract_address).or_default();

        contract_mocked_functions.insert(EntryPointSelector(function_selector.into_()), ret_data);
    }

    pub fn stop_mock_call(
        &mut self,
        contract_address: ContractAddress,
        function_selector: Felt252,
    ) {
        if let std::collections::hash_map::Entry::Occupied(mut e) =
            self.mocked_functions.entry(contract_address)
        {
            let contract_mocked_functions = e.get_mut();
            contract_mocked_functions.remove(&EntryPointSelector(function_selector.into_()));
        }
    }
}
