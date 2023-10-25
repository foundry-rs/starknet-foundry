use crate::CheatnetState;
use cairo_felt::Felt252;
use starknet_api::core::ContractAddress;

impl CheatnetState {
    pub fn start_warp(&mut self, contract_address: ContractAddress, timestamp: Felt252) {
        self.warped_contracts.insert(contract_address, timestamp);
    }

    pub fn stop_warp(&mut self, contract_address: ContractAddress) {
        self.warped_contracts.remove(&contract_address);
    }

    pub fn start_warp_global(&mut self, timestamp: Felt252) {
        self.global_warp = Some(timestamp);
    }

    pub fn stop_warp_global(&mut self) {
        self.global_warp = None;
    }
}
