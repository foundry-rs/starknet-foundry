use crate::state::{CheatSpan, CheatStatus};
use crate::CheatnetState;
use num_traits::Zero;
use starknet_api::core::{ContractAddress, EntryPointSelector};
use starknet_crypto::poseidon_hash_many;
use starknet_types_core::felt::Felt;
use std::collections::hash_map::Entry;

impl CheatnetState {
    pub fn mock_call(
        &mut self,
        contract_address: ContractAddress,
        function_selector: EntryPointSelector,
        call_data: Option<Vec<Felt>>,
        ret_data: &[Felt],
        span: CheatSpan,
    ) {
        let contract_mocked_functions = self.mocked_functions.entry(contract_address).or_default();
        let call_data_hash = match call_data {
            Some(data) => poseidon_hash_many(data.iter()),
            None => Felt::zero(),
        };
        let key = (function_selector, call_data_hash);
        contract_mocked_functions.insert(key, CheatStatus::Cheated(ret_data.to_vec(), span));
    }

    pub fn start_mock_call(
        &mut self,
        contract_address: ContractAddress,
        function_selector: EntryPointSelector,
        call_data: Option<Vec<Felt>>,
        ret_data: &[Felt],
    ) {
        self.mock_call(
            contract_address,
            function_selector,
            call_data,
            ret_data,
            CheatSpan::Indefinite,
        );
    }

    pub fn stop_mock_call(
        &mut self,
        contract_address: ContractAddress,
        function_selector: EntryPointSelector,
        call_data: Option<Vec<Felt>>,
    ) {
        if let Entry::Occupied(mut e) = self.mocked_functions.entry(contract_address) {
            let contract_mocked_functions = e.get_mut();
            let call_data_hash = match call_data {
                Some(data) => poseidon_hash_many(data.iter()),
                None => Felt::zero(),
            };
            contract_mocked_functions.remove(&(function_selector, call_data_hash));
        }
    }
}
