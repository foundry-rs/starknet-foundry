use crate::state::{CheatStatus, CheatTarget};
use crate::CheatnetState;
use cairo_felt::Felt252;

impl CheatnetState {
    pub fn start_roll(&mut self, target: CheatTarget, block_number: Felt252) {
        match target {
            CheatTarget::All => {
                self.global_roll = Some(block_number);
                self.rolled_contracts.clear();
            }
            CheatTarget::One(contract_address) => {
                self.rolled_contracts
                    .insert(contract_address, CheatStatus::Cheated(block_number));
            }
            CheatTarget::Multiple(contracts) => {
                for contract_address in contracts {
                    self.rolled_contracts
                        .insert(contract_address, CheatStatus::Cheated(block_number.clone()));
                }
            }
        }
    }

    pub fn stop_roll(&mut self, target: CheatTarget) {
        match target {
            CheatTarget::All => {
                self.global_roll = None;
                self.rolled_contracts.clear();
            }
            CheatTarget::One(contract_address) => {
                self.rolled_contracts
                    .insert(contract_address, CheatStatus::Uncheated);
            }
            CheatTarget::Multiple(contracts) => {
                for contract_address in contracts {
                    self.rolled_contracts
                        .insert(contract_address, CheatStatus::Uncheated);
                }
            }
        }
    }
}
