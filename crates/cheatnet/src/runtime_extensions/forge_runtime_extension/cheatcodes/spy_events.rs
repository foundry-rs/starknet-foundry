use crate::CheatnetState;
use blockifier::execution::call_info::OrderedEvent;
use conversions::{serde::serialize::CairoSerialize, FromConv};
use starknet_api::core::ContractAddress;
use starknet_types_core::felt::Felt;

/// Represents an emitted event. It is used in the `CheatnetState` to keep track of events
/// emitted in the `cheatnet::src::rpc::call_contract`
#[derive(CairoSerialize, Debug, PartialEq, Clone)]
pub struct Event {
    pub from: ContractAddress,
    pub keys: Vec<Felt>,
    pub data: Vec<Felt>,
}

impl Event {
    #[must_use]
    pub fn from_ordered_event(
        ordered_event: &OrderedEvent,
        contract_address: ContractAddress,
    ) -> Self {
        Self {
            from: contract_address,
            keys: ordered_event
                .event
                .keys
                .iter()
                .map(|key| Felt::from_(key.0))
                .collect(),
            data: ordered_event
                .event
                .data
                .0
                .iter()
                .map(|el| Felt::from_(*el))
                .collect(),
        }
    }
}

impl CheatnetState {
    pub fn get_events(&mut self, event_offset: usize) -> Vec<Event> {
        self.detected_events[event_offset..].to_vec()
    }
}
