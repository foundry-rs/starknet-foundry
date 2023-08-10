use cairo_felt::Felt252;
use starknet_api::core::ContractAddress;
use std::collections::HashMap;

pub mod cheatcodes;
pub mod constants;
pub mod conversions;
pub mod panic_data;
pub mod rpc;
pub mod state;

pub struct CheatedState {
    pub rolled_contracts: HashMap<ContractAddress, Felt252>,
    pub pranked_contracts: HashMap<ContractAddress, ContractAddress>,
    pub warped_contracts: HashMap<ContractAddress, Felt252>,
}

impl CheatedState {
    #[must_use]
    pub fn new() -> Self {
        CheatedState {
            rolled_contracts: HashMap::new(),
            pranked_contracts: HashMap::new(),
            warped_contracts: HashMap::new(),
        }
    }
}

impl Default for CheatedState {
    fn default() -> Self {
        Self::new()
    }
}
