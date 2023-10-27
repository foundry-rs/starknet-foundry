use crate::CheatnetState;
use crate::state::CheatTarget;
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
                self.warped_contracts.insert(contract_address, timestamp);
            }
            CheatTarget::Multiple(contracts) => {
                for contract_address in contracts {
                    self.warped_contracts.insert(contract_address, timestamp.clone());
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
            // TODO: Fix this logic so it works even after `All` warp
            CheatTarget::One(contract_address) => {
                self.warped_contracts.remove(&contract_address);
            }
            CheatTarget::Multiple(contracts) => {
                for contract_address in contracts {
                    self.warped_contracts.remove(&contract_address);
                }
            }
        }
    }
}
