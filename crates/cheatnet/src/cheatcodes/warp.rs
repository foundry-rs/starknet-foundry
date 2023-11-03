use crate::state::{CheatStatus, CheatTarget};
use crate::CheatnetState;
use cairo_felt::Felt252;

impl CheatnetState {
    pub fn start_warp(&mut self, target: CheatTarget, timestamp: Felt252) {
        match target {
            CheatTarget::All => {
                self.global_warp = Some(timestamp);
                // Clear individual warps so that `All`
                // contracts are affected by this warp
                self.warped_contracts.clear();
            }
            CheatTarget::One(contract_address) => {
                self.warped_contracts
                    .insert(contract_address, CheatStatus::Cheated(timestamp));
            }
            CheatTarget::Multiple(contract_addresses) => {
                for contract_address in contract_addresses {
                    self.warped_contracts
                        .insert(contract_address, CheatStatus::Cheated(timestamp.clone()));
                }
            }
        }
    }

    pub fn stop_warp(&mut self, target: CheatTarget) {
        match target {
            CheatTarget::All => {
                self.global_warp = None;
                self.warped_contracts.clear();
            }
            CheatTarget::One(contract_address) => {
                self.warped_contracts
                    .insert(contract_address, CheatStatus::Uncheated);
            }
            CheatTarget::Multiple(contract_addresses) => {
                for contract_address in contract_addresses {
                    self.warped_contracts
                        .insert(contract_address, CheatStatus::Uncheated);
                }
            }
        }
    }
}
