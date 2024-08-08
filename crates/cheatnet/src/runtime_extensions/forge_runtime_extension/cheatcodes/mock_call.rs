use crate::state::{CheatSpan, CheatStatus};
use crate::CheatnetState;
use cairo_vm::Felt252;
use starknet_api::core::{ContractAddress, EntryPointSelector};
use std::collections::hash_map::Entry;

impl CheatnetState {
    pub fn mock_call(
        &mut self,
        contract_address: ContractAddress,
        function_selector: EntryPointSelector,
        ret_data: &[Felt252],
        span: CheatSpan,
    ) {
        let contract_mocked_functions = self.mocked_functions.entry(contract_address).or_default();

        contract_mocked_functions.insert(
            function_selector,
            CheatStatus::Cheated(ret_data.to_vec(), span),
        );
    }

    pub fn start_mock_call(
        &mut self,
        contract_address: ContractAddress,
        function_selector: EntryPointSelector,
        ret_data: &[Felt252],
    ) {
        self.mock_call(
            contract_address,
            function_selector,
            ret_data,
            CheatSpan::Indefinite,
        );
    }

    pub fn stop_mock_call(
        &mut self,
        contract_address: ContractAddress,
        function_selector: EntryPointSelector,
    ) {
        if let Entry::Occupied(mut e) = self.mocked_functions.entry(contract_address) {
            let contract_mocked_functions = e.get_mut();
            contract_mocked_functions.remove(&function_selector);
        }
    }
}
