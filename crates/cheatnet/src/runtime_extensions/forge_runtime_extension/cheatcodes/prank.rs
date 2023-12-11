use crate::state::{start_cheat, stop_cheat, CheatTarget};
use crate::CheatnetState;
use starknet_api::core::ContractAddress;

impl CheatnetState {
    pub fn start_prank(&mut self, target: CheatTarget, caller_address: ContractAddress) {
        start_cheat(
            &mut self.global_prank,
            &mut self.pranked_contracts,
            target,
            caller_address,
        );
    }

    pub fn stop_prank(&mut self, target: CheatTarget) {
        stop_cheat(&mut self.global_prank, &mut self.pranked_contracts, target);
    }
}
