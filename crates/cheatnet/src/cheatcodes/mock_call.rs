use crate::CheatnetState;
use blockifier::abi::abi_utils::selector_from_name;
use blockifier::execution::execution_utils::felt_to_stark_felt;
use cairo_felt::Felt252;
use cairo_lang_runner::short_string::as_cairo_short_string;
use starknet_api::core::ContractAddress;
use starknet_api::hash::StarkFelt;

impl CheatnetState {
    pub fn start_mock_call(
        &mut self,
        contract_address: ContractAddress,
        function_name: &Felt252,
        ret_data: &[Felt252],
    ) {
        let ret_data: Vec<StarkFelt> = ret_data.iter().map(felt_to_stark_felt).collect();

        let function_name = selector_from_name(&as_cairo_short_string(function_name).unwrap());

        let contract_mocked_functions = self.mocked_functions.entry(contract_address).or_default();

        contract_mocked_functions.insert(function_name, ret_data);
    }

    pub fn stop_mock_call(&mut self, contract_address: ContractAddress, function_name: &Felt252) {
        let function_name = selector_from_name(&as_cairo_short_string(function_name).unwrap());

        if let std::collections::hash_map::Entry::Occupied(mut e) =
            self.mocked_functions.entry(contract_address)
        {
            let contract_mocked_functions = e.get_mut();
            contract_mocked_functions.remove(&function_name);
        }
    }
}
