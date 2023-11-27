use crate::state::{start_cheat, stop_cheat, CheatTarget};
use crate::CheatnetState;
use starknet_api::core::ContractAddress;

impl CheatnetState {
    pub fn start_elect(&mut self, target: CheatTarget, sequencer_address: ContractAddress) {
        start_cheat(
            &mut self.global_elect,
            &mut self.elected_contracts,
            target,
            sequencer_address,
        );
    }

    pub fn stop_elect(&mut self, target: CheatTarget) {
        stop_cheat(&mut self.global_elect, &mut self.elected_contracts, target);
    }
}
