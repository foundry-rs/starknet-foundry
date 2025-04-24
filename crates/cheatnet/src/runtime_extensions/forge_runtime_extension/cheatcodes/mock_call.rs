use crate::CheatnetState;
use crate::state::{CheatSpan, CheatStatus};
use starknet_api::core::{ContractAddress, EntryPointSelector};
use starknet_types_core::felt::Felt;
use std::collections::hash_map::Entry;

impl CheatnetState {
    pub fn mock_call(
        &mut self,
        contract_address: ContractAddress,
        function_selector: EntryPointSelector,
        ret_data: &[Felt],
        span: CheatSpan,
    ) {
        if let CheatSpan::TargetCalls(n) = span {
            if n == 0 {
                panic!("CheatSpan::TargetCalls(0) is not allowed");
            }
        }
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
        ret_data: &[Felt],
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
