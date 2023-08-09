use crate::{cheatcodes::EnhancedHintError, CheatedState};
use starknet_api::core::ContractAddress;

impl CheatedState {
    pub fn start_prank(
        &mut self,
        contract_address: ContractAddress,
        caller_address: ContractAddress,
    ) -> Result<(), EnhancedHintError> {
        self.pranked_contracts
            .insert(contract_address, caller_address);
        Ok(())
    }

    pub fn stop_prank(
        &mut self,
        contract_address: ContractAddress,
    ) -> Result<(), EnhancedHintError> {
        self.pranked_contracts.remove(&contract_address);
        Ok(())
    }
}
