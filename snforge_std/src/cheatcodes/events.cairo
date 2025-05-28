use starknet::ContractAddress;
use crate::cheatcode::execute_cheatcode_and_deserialize;


/// Creates `EventSpy` instance that spies on all events emitted after its creation.
pub fn spy_events() -> EventSpy {
    execute_cheatcode_and_deserialize::<'spy_events'>(array![].span())
}

/// Raw event format (as seen via the RPC-API), can be used for asserting the emitted events.
#[derive(Drop, Clone, Serde, Debug, PartialEq)]
pub struct Event {
    pub keys: Array<felt252>,
    pub data: Array<felt252>,
}

/// An event spy structure allowing to get events emitted only after its creation.
#[derive(Drop, Serde)]
pub struct EventSpy {
    event_offset: usize,
}

/// A wrapper structure on an array of events to handle filtering smoothly.
#[derive(Drop, Serde, Clone, Debug, PartialEq)]
pub struct Events {
    pub events: Array<(ContractAddress, Event)>,
}

pub trait EventSpyTrait {
    /// Gets all events given [`EventSpy`] spies for.
    fn get_events(ref self: EventSpy) -> Events;
}

impl EventSpyTraitImpl of EventSpyTrait {
    fn get_events(ref self: EventSpy) -> Events {
        execute_cheatcode_and_deserialize::<'get_events'>(array![self.event_offset.into()].span())
    }
}

pub trait EventsFilterTrait {
    /// Filter events emitted by a given [`ContractAddress`].
    fn emitted_by(self: @Events, contract_address: ContractAddress) -> Events;
}

impl EventsFilterTraitImpl of EventsFilterTrait {
    fn emitted_by(self: @Events, contract_address: ContractAddress) -> Events {
        let mut new_events = array![];

        for (from, event) in self.events.span() {
            if *from == contract_address {
                new_events.append((*from, event.clone()));
            };
        }
        Events { events: new_events }
    }
}

/// Allows to assert the expected events emission (or lack thereof),
/// in the scope of [`EventSpy`] structure.
pub trait EventSpyAssertionsTrait<T, +starknet::Event<T>, +Drop<T>> {
    fn assert_emitted(ref self: EventSpy, events: @Array<(ContractAddress, T)>);
    fn assert_not_emitted(ref self: EventSpy, events: @Array<(ContractAddress, T)>);
}

impl EventSpyAssertionsTraitImpl<T, +starknet::Event<T>, +Drop<T>> of EventSpyAssertionsTrait<T> {
    fn assert_emitted(ref self: EventSpy, events: @Array<(ContractAddress, T)>) {
        let received_events = self.get_events();

        for (from, event) in events.span() {
            if !received_events.is_emitted(*from, event) {
                let from: felt252 = (*from).into();
                panic!("Event with matching data and keys was not emitted from {}", from);
            }
        };
    }

    fn assert_not_emitted(ref self: EventSpy, events: @Array<(ContractAddress, T)>) {
        let received_events = self.get_events();

        for (from, event) in events.span() {
            if received_events.is_emitted(*from, event) {
                let from: felt252 = (*from).into();
                panic!("Event with matching data and keys was emitted from {}", from);
            }
        };
    }
}

pub trait IsEmitted<T, +starknet::Event<T>, +Drop<T>> {
    fn is_emitted(self: @Events, expected_emitted_by: ContractAddress, expected_event: @T) -> bool;
}

pub impl IsEmittedImpl<T, +starknet::Event<T>, +Drop<T>> of IsEmitted<T> {
    fn is_emitted(self: @Events, expected_emitted_by: ContractAddress, expected_event: @T) -> bool {
        let expected_event: Event = expected_event.into();

        let mut is_emitted = false;
        for (from, event) in self.events.span() {
            if from == @expected_emitted_by && event == @expected_event {
                is_emitted = true;
                break;
            };
        }
        return is_emitted;
    }
}


impl EventIntoImpl<T, +starknet::Event<T>, +Drop<T>> of Into<T, Event> {
    fn into(self: T) -> Event {
        let mut keys = array![];
        let mut data = array![];
        self.append_keys_and_data(ref keys, ref data);
        Event { keys, data }
    }
}

impl EventSnapIntoImpl<T, +starknet::Event<T>, +Drop<T>> of Into<@T, Event> {
    fn into(self: @T) -> Event {
        let mut keys = array![];
        let mut data = array![];
        self.append_keys_and_data(ref keys, ref data);
        Event { keys, data }
    }
}

impl EventTraitImpl of starknet::Event<Event> {
    fn append_keys_and_data(self: @Event, ref keys: Array<felt252>, ref data: Array<felt252>) {
        keys.append_span(self.keys.span());
        data.append_span(self.data.span());
    }
    fn deserialize(ref keys: Span<felt252>, ref data: Span<felt252>) -> Option<Event> {
        Option::None
    }
}
