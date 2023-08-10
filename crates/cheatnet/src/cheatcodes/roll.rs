use crate::CheatedState;
use cairo_felt::Felt252;
use starknet_api::core::ContractAddress;

impl CheatedState {
    pub fn start_roll(&mut self, contract_address: ContractAddress, block_number: Felt252) {
        self.rolled_contracts.insert(contract_address, block_number);
    }

    pub fn stop_roll(&mut self, contract_address: ContractAddress) {
        self.rolled_contracts.remove(&contract_address);
    }
}
