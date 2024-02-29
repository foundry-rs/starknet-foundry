use crate::state::{start_cheat, stop_cheat, CheatSpan, CheatTarget};
use crate::CheatnetState;
use starknet_api::core::ContractAddress;

impl CheatnetState {
    pub fn prank(&mut self, target: CheatTarget, caller_address: ContractAddress, span: CheatSpan) {
        start_cheat(
            &mut self.global_prank,
            &mut self.pranked_contracts,
            target,
            caller_address,
            span,
        );
    }

    pub fn start_prank(&mut self, target: CheatTarget, caller_address: ContractAddress) {
        self.prank(target, caller_address, CheatSpan::Indefinite);
    }

    pub fn stop_prank(&mut self, target: CheatTarget) {
        stop_cheat(&mut self.global_prank, &mut self.pranked_contracts, target);
    }
}
