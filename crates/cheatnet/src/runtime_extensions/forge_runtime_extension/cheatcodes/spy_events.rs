use crate::CheatnetState;
use blockifier::execution::call_info::OrderedEvent;
use cairo_felt::Felt252;
use cairo_vm::hint_processor::hint_processor_utils::felt_to_usize;
use conversions::FromConv;
use starknet_api::core::ContractAddress;

/// Represents an emitted event. It is used in the `CheatnetState` to keep track of events
/// emitted in the `cheatnet::src::rpc::call_contract`
#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug)]
pub struct Spy {
    pub(crate) target: SpyTarget,
    pub(crate) events: Vec<Event>,
}

impl Spy {
    #[must_use]
    pub fn new(target: SpyTarget) -> Self {
        Self {
            target,
            events: Vec::new(),
        }
    }
}

/// Specifies which contract are spied on.
#[derive(Debug)]
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

pub struct Events {
    events: Vec<Felt252>,
}

#[allow(clippy::len_without_is_empty)]
impl Events {
    #[must_use]
    pub fn events(&self) -> &[Felt252] {
        &self.events[1..]
    }

    #[must_use]
    pub fn len(&self) -> usize {
        felt_to_usize(&self.events[0]).unwrap()
    }
}

impl From<Events> for Vec<Felt252> {
    fn from(value: Events) -> Self {
        value.events
    }
}

impl CheatnetState {
    pub fn spy_events(&mut self, spy_on: SpyTarget) -> usize {
        self.spies.push(Spy::new(spy_on));
        self.spies.len() - 1
    }

    pub fn fetch_events(&mut self, id: &Felt252) -> Events {
        let spy = &mut self.spies[felt_to_usize(id).unwrap()];

        let mut serialized_events = vec![Felt252::from(spy.events.len())];

        for event in spy.events.drain(..) {
            serialized_events.push(Felt252::from_(event.from));
            serialized_events.push(Felt252::from(event.keys.len()));
            serialized_events.extend(event.keys);
            serialized_events.push(Felt252::from(event.data.len()));
            serialized_events.extend(event.data);
        }

        Events {
            events: serialized_events,
        }
    }
}
