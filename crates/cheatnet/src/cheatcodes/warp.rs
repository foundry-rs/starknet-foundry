use crate::{cheatcodes::EnhancedHintError, CheatedState};
use cairo_felt::Felt252;
use starknet_api::core::ContractAddress;

impl CheatedState {
    pub fn start_warp(
        &mut self,
        contract_address: ContractAddress,
        timestamp: Felt252,
    ) -> Result<(), EnhancedHintError> {
        self.warped_contracts.insert(contract_address, timestamp);
        Ok(())
    }

    pub fn stop_warp(
        &mut self,
        contract_address: ContractAddress,
    ) -> Result<(), EnhancedHintError> {
        self.warped_contracts.remove(&contract_address);
        Ok(())
    }
}
