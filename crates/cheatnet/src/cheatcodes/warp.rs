use crate::CheatnetState;
use cairo_felt::Felt252;
use starknet_api::core::ContractAddress;

// Specifies which contracts to warp
#[derive(Debug)]
pub enum StartWarpTarget {
    All(Felt252),
    One((ContractAddress, Felt252)),
    Multiple(Vec<(ContractAddress, Felt252)>),
}

// Specifies which contracts to stop warping
#[derive(Debug)]
pub enum StopWarpTarget {
    All,
    One(ContractAddress),
    Multiple(Vec<ContractAddress>),
}

impl CheatnetState {
    pub fn start_warp(&mut self, warp_target: StartWarpTarget) {
        match warp_target {
            StartWarpTarget::All(timestamp) => {
                self.global_warp = Some(timestamp);
                // Clear individual warps so that `All`
                // contracts are affected by this warp
                self.warped_contracts.clear();
            }
            StartWarpTarget::One((contract_address, timestamp)) => {
                self.warped_contracts.insert(contract_address, timestamp);
            }
            StartWarpTarget::Multiple(contracts) => {
                for (contract_address, timestamp) in contracts {
                    self.warped_contracts.insert(contract_address, timestamp);
                }
            }
        }
    }

    pub fn stop_warp(&mut self, warp_target: StopWarpTarget) {
        match warp_target {
            StopWarpTarget::All => {
                self.global_warp = None;
                self.warped_contracts.clear();
            }
            StopWarpTarget::One(contract_address) => {
                self.warped_contracts.remove(&contract_address);
            }
            StopWarpTarget::Multiple(contracts) => {
                for contract_address in contracts {
                    self.warped_contracts.remove(&contract_address);
                }
            }
        }
    }
}
