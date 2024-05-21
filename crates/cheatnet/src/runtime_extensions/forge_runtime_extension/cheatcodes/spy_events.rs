use crate::CheatnetState;
use blockifier::execution::call_info::OrderedEvent;
use cairo_felt::Felt252;
use cairo_vm::hint_processor::hint_processor_utils::felt_to_usize;
use conversions::{
    serde::{deserialize::CairoDeserialize, serialize::CairoSerialize},
    FromConv,
};
use starknet_api::core::ContractAddress;
use std::mem::take;

/// Represents an emitted event. It is used in the `CheatnetState` to keep track of events
/// emitted in the `cheatnet::src::rpc::call_contract`
#[derive(CairoSerialize, Debug, PartialEq, Clone)]
pub struct Event {
    pub from: ContractAddress,
    pub keys: Vec<Felt252>,
    pub data: Vec<Felt252>,
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
                .map(|key| Felt252::from_(key.0))
                .collect(),
            data: ordered_event
                .event
                .data
                .0
                .iter()
                .map(|el| Felt252::from_(*el))
                .collect(),
        }
    }
}

/// Specifies which contract are spied on.
#[derive(CairoDeserialize, Debug)]
pub enum SpyTarget {
    All,
    One(ContractAddress),
    Multiple(Vec<ContractAddress>),
}

impl SpyTarget {
    #[must_use]
    pub fn does_spy(&self, contract_address: ContractAddress) -> bool {
        match self {
            SpyTarget::All => true,
            SpyTarget::One(address) => *address == contract_address,
            SpyTarget::Multiple(addresses) => addresses.contains(&contract_address),
        }
    }
}

impl CheatnetState {
    pub fn spy_events(&mut self, spy_on: SpyTarget) -> usize {
        self.spies.push(spy_on);
        self.spies.len() - 1
    }

    pub fn fetch_events(&mut self, id: &Felt252) -> Vec<Event> {
        let spy_on = &self.spies[felt_to_usize(id).unwrap()];

        // replace with empty to get ownership
        let emitted_events = take(&mut self.detected_events);

        emitted_events
            .into_iter()
            .filter_map(|event| {
                if spy_on.does_spy(event.from) {
                    Some(event)
                } else {
                    // push back unconsumed ones
                    self.detected_events.push(event);

                    None
                }
            })
            .collect()
    }
}
