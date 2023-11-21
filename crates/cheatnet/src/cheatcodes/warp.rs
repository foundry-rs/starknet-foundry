use crate::state::{start_cheat, stop_cheat, CheatTarget};
use crate::CheatnetState;
use cairo_felt::Felt252;

impl CheatnetState {
    pub fn start_warp(&mut self, target: CheatTarget, timestamp: Felt252) {
        start_cheat(
            &mut self.global_warp,
            &mut self.warped_contracts,
            target,
            timestamp,
        );
    }

    pub fn stop_warp(&mut self, target: CheatTarget) {
        stop_cheat(&mut self.global_warp, &mut self.warped_contracts, target);
    }
}
