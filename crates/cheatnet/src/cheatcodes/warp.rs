use crate::CheatnetState;
use cairo_felt::Felt252;
use starknet_api::core::ContractAddress;

impl CheatnetState {
    pub fn start_warp(&mut self, contract_address: ContractAddress, timestamp: Felt252) {
        self.cheatcode_state
            .warped_contracts
            .insert(contract_address, timestamp);
    }

    pub fn stop_warp(&mut self, contract_address: ContractAddress) {
        self.cheatcode_state
            .warped_contracts
            .remove(&contract_address);
    }
}
