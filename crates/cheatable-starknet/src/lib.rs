use cairo_felt::Felt252;
use starknet_api::core::ContractAddress;
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::ContractAddressSalt;
use std::collections::HashMap;

pub mod cheatcodes;
pub mod constants;
pub mod rpc;
pub mod state;

pub struct CheatedState {
    pub rolled_contracts: HashMap<ContractAddress, Felt252>,
    pub pranked_contracts: HashMap<ContractAddress, ContractAddress>,
    pub warped_contracts: HashMap<ContractAddress, Felt252>,
    pub deploy_counter: u32,
}

impl CheatedState {
    #[must_use]
    pub fn new() -> Self {
        CheatedState {
            rolled_contracts: HashMap::new(),
            pranked_contracts: HashMap::new(),
            warped_contracts: HashMap::new(),
            deploy_counter: 0,
        }
    }

    pub fn increment_deploy_counter(&mut self) -> () {
        self.deploy_counter += 1;
    }

    pub fn gen_salt(&self) -> ContractAddressSalt {
        ContractAddressSalt(StarkFelt::from(333_u32 + self.deploy_counter))
    }
}

impl Default for CheatedState {
    fn default() -> Self {
        Self::new()
    }
}
