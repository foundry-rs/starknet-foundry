use crate::state::{start_cheat, stop_cheat, CheatSpan, CheatTarget};
use crate::CheatnetState;
use cairo_felt::Felt252;

impl CheatnetState {
    pub fn warp(&mut self, target: CheatTarget, timestamp: Felt252, span: CheatSpan) {
        start_cheat(
            &mut self.global_warp,
            &mut self.warped_contracts,
            target,
            timestamp,
            span,
        );
    }

    pub fn start_warp(&mut self, target: CheatTarget, timestamp: Felt252) {
        self.warp(target, timestamp, CheatSpan::Indefinite);
    }

    pub fn stop_warp(&mut self, target: CheatTarget) {
        stop_cheat(&mut self.global_warp, &mut self.warped_contracts, target);
    }
}
