use crate::state::{CheatStatus, CheatTarget};
use crate::CheatnetState;
use starknet_api::core::ContractAddress;

impl CheatnetState {
    pub fn start_prank(&mut self, target: CheatTarget, caller_address: ContractAddress) {
        match target {
            CheatTarget::All => {
                self.global_prank = Some(caller_address);
                // Clear individual pranks so that `All`
                // contracts are affected by this prank
                self.pranked_contracts.clear();
            }
            CheatTarget::One(contract_address) => {
                self.pranked_contracts
                    .insert(contract_address, CheatStatus::Cheated(caller_address));
            }
            CheatTarget::Multiple(contract_addresses) => {
                for contract_address in contract_addresses {
                    self.pranked_contracts
                        .insert(contract_address, CheatStatus::Cheated(caller_address));
                }
            }
        }
    }

    pub fn stop_prank(&mut self, target: CheatTarget) {
        match target {
            CheatTarget::All => {
                self.global_prank = None;
                self.pranked_contracts.clear();
            }
            CheatTarget::One(contract_address) => {
                self.pranked_contracts
                    .insert(contract_address, CheatStatus::Uncheated);
            }
            CheatTarget::Multiple(contract_addresses) => {
                for contract_address in contract_addresses {
                    self.pranked_contracts
                        .insert(contract_address, CheatStatus::Uncheated);
                }
            }
        }
    }
}
