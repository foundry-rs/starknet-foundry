use crate::CheatnetState;
use cairo_felt::Felt252;
use cairo_vm::hint_processor::hint_processor_utils::felt_to_usize;
use starknet_api::core::ContractAddress;

/// Represents an emitted event. It is used in the `CheatnetState` to keep track of events
/// emitted in the `cheatnet::src::rpc::call_contract`
#[derive(Debug, PartialEq, Clone)]
pub enum Event {
    Named(NamedEvent),
    Unnamed(UnnamedEvent),
}

impl Event {
    #[must_use]
    pub fn from(&self) -> ContractAddress {
        match self {
            Event::Unnamed(UnnamedEvent { from, .. }) | Event::Named(NamedEvent { from, .. }) => {
                *from
            }
        }
    }

    #[must_use]
    pub fn name(&self) -> Option<Felt252> {
        match self {
            Event::Named(NamedEvent { name, .. }) => Some(name.clone()),
            Event::Unnamed(_) => None,
        }
    }

    #[must_use]
    pub fn keys(&self) -> Vec<Felt252> {
        match self {
            Event::Unnamed(UnnamedEvent { keys, .. }) | Event::Named(NamedEvent { keys, .. }) => {
                keys.clone()
            }
        }
    }

    #[must_use]
    pub fn data(&self) -> Vec<Felt252> {
        match self {
            Event::Unnamed(UnnamedEvent { data, .. }) | Event::Named(NamedEvent { data, .. }) => {
                data.clone()
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct NamedEvent {
    pub from: ContractAddress,
    pub name: Felt252,
    pub keys: Vec<Felt252>,
    pub data: Vec<Felt252>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct UnnamedEvent {
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
                if spy_on.does_spy(event.from()) {
                    match event {
                        Event::Named(_) => flattened_event.push(Felt252::from(0)),
                        Event::Unnamed(_) => flattened_event.push(Felt252::from(1)),
                    };
                    flattened_event.push(Felt252::from_bytes_be(event.from().0.key().bytes()));
                    if let Some(name) = event.name() {
                        flattened_event.append(&mut vec![name.clone()]);
                    };
                    flattened_event.push(Felt252::from(event.keys().len()));
                    flattened_event.append(&mut event.keys().clone());
                    flattened_event.push(Felt252::from(event.data().len()));
                    flattened_event.append(&mut event.data().clone());

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
