use crate::cheatcodes::spy_events::Event;
use cairo_felt::Felt252;
use starknet_api::core::ContractAddress;
use std::collections::HashMap;

pub mod cheatcodes;
pub mod constants;
pub mod rpc;
pub mod state;

pub struct CheatedState {
    pub rolled_contracts: HashMap<ContractAddress, Felt252>,
    pub pranked_contracts: HashMap<ContractAddress, ContractAddress>,
    pub warped_contracts: HashMap<ContractAddress, Felt252>,
    pub spy_events: bool,
    pub emitted_events: Vec<Event>,
}

impl CheatedState {
    #[must_use]
    pub fn new() -> Self {
        CheatedState {
            rolled_contracts: HashMap::new(),
            pranked_contracts: HashMap::new(),
            warped_contracts: HashMap::new(),
            spy_events: false,
            emitted_events: vec![],
        }
    }
}

impl Default for CheatedState {
    fn default() -> Self {
        Self::new()
    }
}
