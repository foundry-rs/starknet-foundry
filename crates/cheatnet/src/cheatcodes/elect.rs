use crate::CheatnetState;
use starknet_api::core::ContractAddress;

impl CheatnetState {
    pub fn start_elect(&mut self, contract_address: ContractAddress, sequencer_address: ContractAddress) {
        self.elected_contracts.insert(contract_address, sequencer_address);
    }

    pub fn stop_elect(&mut self, contract_address: ContractAddress) {
        self.elected_contracts.remove(&contract_address);
    }
}
