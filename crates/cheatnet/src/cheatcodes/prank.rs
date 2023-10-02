use crate::CheatnetState;
use starknet_api::core::ContractAddress;

impl CheatnetState {
    pub fn start_prank(
        &mut self,
        contract_address: ContractAddress,
        caller_address: ContractAddress,
    ) {
        self.pranked_contracts
            .insert(contract_address, caller_address);
    }

    pub fn stop_prank(&mut self, contract_address: ContractAddress) {
        self.pranked_contracts.remove(&contract_address);
    }
}
