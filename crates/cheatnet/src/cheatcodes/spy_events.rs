use crate::CheatnetState;
use cairo_felt::Felt252;
use cairo_vm::hint_processor::hint_processor_utils::felt_to_usize;
use conversions::StarknetConversions;
use starknet_api::core::ContractAddress;

/// Represents an emitted event. It is used in the `CheatnetState` to keep track of events
/// emitted in the `cheatnet::src::rpc::call_contract`
#[derive(Debug, PartialEq, Clone)]
pub struct Event {
    pub from: ContractAddress,
    pub keys: Vec<Felt252>,
    pub data: Vec<Felt252>,
}

/// Specifies which contract are spied on.
pub enum SpyTarget {
    All,
    One(ContractAddress),
    Multiple(Vec<ContractAddress>),
}

impl SpyTarget {
    pub fn does_spy(&mut self, contract_address: ContractAddress) -> bool {
        match self {
            SpyTarget::All => true,
            SpyTarget::One(address) => *address == contract_address,
            SpyTarget::Multiple(addresses) => addresses.contains(&contract_address),
        }
    }
}

impl CheatnetState {
    pub fn spy_events(&mut self, spy_on: SpyTarget) -> usize {
        self.cheatcode_state.spies.push(spy_on);
        self.cheatcode_state.spies.len() - 1
    }

    pub fn fetch_events(&mut self, id: &Felt252) -> (usize, Vec<Felt252>) {
        let spy_on = &mut self.cheatcode_state.spies[felt_to_usize(id).unwrap()];
        let mut spied_events_len = 0;
        let mut unconsumed_emitted_events: Vec<Event> = vec![];

        let serialized_events: Vec<Vec<Felt252>> = self
            .cheatcode_state
            .detected_events
            .iter()
            .map(|event| {
                let mut flattened_event = vec![];
                if spy_on.does_spy(event.from) {
                    flattened_event.push(event.from.to_felt252());
                    flattened_event.push(Felt252::from(event.keys.len()));
                    flattened_event.append(&mut event.keys.clone());
                    flattened_event.push(Felt252::from(event.data.len()));
                    flattened_event.append(&mut event.data.clone());

                    spied_events_len += 1;
                } else {
                    unconsumed_emitted_events.push(event.clone());
                }
                flattened_event
            })
            .collect();

        self.cheatcode_state.detected_events = unconsumed_emitted_events;
        (spied_events_len, serialized_events.concat())
    }
}
