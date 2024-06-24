use crate::state::{start_cheat, stop_cheat, CheatSpan, CheatTarget};
use crate::CheatnetState;
use starknet_api::core::ContractAddress;

impl CheatnetState {
    pub fn elect(
        &mut self,
        target: CheatTarget,
        sequencer_address: ContractAddress,
        span: CheatSpan,
    ) {
        start_cheat(
            &mut self.global_elect,
            &mut self.elected_contracts,
            target,
            sequencer_address,
            span,
        );
    }

    pub fn start_elect(&mut self, target: CheatTarget, sequencer_address: ContractAddress) {
        self.elect(target, sequencer_address, CheatSpan::Indefinite);
    }

    pub fn stop_elect(&mut self, target: CheatTarget) {
        stop_cheat(&mut self.global_elect, &mut self.elected_contracts, target);
    }
}
