use crate::state::{start_cheat, stop_cheat, CheatTarget};
use crate::CheatnetState;
use cairo_felt::Felt252;

impl CheatnetState {
    pub fn start_roll(&mut self, target: CheatTarget, block_number: Felt252) {
        start_cheat(
            &mut self.global_roll,
            &mut self.rolled_contracts,
            target,
            block_number,
        );
    }

    pub fn stop_roll(&mut self, target: CheatTarget) {
        stop_cheat(&mut self.global_roll, &mut self.rolled_contracts, target);
    }
}
