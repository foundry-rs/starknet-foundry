use core::array::ArrayTrait;
use core::option::OptionTrait;
use starknet::testing::cheatcode;
use starknet::ContractAddress;
use super::super::_cheatcode::handle_cheatcode;

/// Creates `EventSpy` instance that spies on all events emitted after its creation.
fn spy_events() -> EventSpy {
    let mut event_offset = handle_cheatcode(cheatcode::<'spy_events'>(array![].span()));
    let parsed_event_offset: usize = Serde::<usize>::deserialize(ref event_offset).unwrap();

    EventSpy { _event_offset: parsed_event_offset }
}

/// Raw event format (as seen via the RPC-API), can be used for asserting the emitted events.
#[derive(Drop, Clone, Serde)]
struct Event {
    keys: Array<felt252>,
    data: Array<felt252>
}

/// An event spy structure allowing to get events emitted only after its creation.
#[derive(Drop, Serde)]
struct EventSpy {
    _event_offset: usize
}

/// A wrapper structure on an array of events to handle filtering smoothly.
#[derive(Drop, Serde)]
struct Events {
    events: Array<(ContractAddress, Event)>
}

trait EventSpyTrait {
    /// Gets all events given [`EventSpy`] spies for.
    fn get_events(ref self: EventSpy) -> Events;
}

impl EventSpyTraitImpl of EventSpyTrait {
    fn get_events(ref self: EventSpy) -> Events {
        let mut output = handle_cheatcode(
            cheatcode::<'get_events'>(array![self._event_offset.into()].span())
        );
        let events = Serde::<Array<(ContractAddress, Event)>>::deserialize(ref output).unwrap();

        Events { events }
    }
}

trait EventsFilterTrait {
    /// Filter events emitted by a given [`ContractAddress`].
    fn emitted_by(self: @Events, contract_address: ContractAddress) -> Events;
}

impl EventsFilterTraitImpl of EventsFilterTrait {
    fn emitted_by(self: @Events, contract_address: ContractAddress) -> Events {
        let mut counter = 0;
        let mut new_events = array![];

        while counter < self.events.len() {
            let (from, event) = self.events.at(counter);
            if *from == contract_address {
                new_events.append((*from, event.clone()));
            };
            counter += 1;
        };
        Events { events: new_events }
    }
}

/// Allows to assert the expected events emission (or lack thereof),
/// in the scope of [`EventSpy`] structure.
trait EventSpyAssertionsTrait<T, impl TEvent: starknet::Event<T>, impl TDrop: Drop<T>> {
    fn assert_emitted(ref self: EventSpy, events: @Array<(ContractAddress, T)>);
    fn assert_not_emitted(ref self: EventSpy, events: @Array<(ContractAddress, T)>);
}

impl EventSpyAssertionsTraitImpl<
    T, impl TEvent: starknet::Event<T>, impl TDrop: Drop<T>
> of EventSpyAssertionsTrait<T> {
    fn assert_emitted(ref self: EventSpy, events: @Array<(ContractAddress, T)>) {
        let mut i = 0;
        let received_events = self.get_events();

        while i < events.len() {
            let (from, event) = events.at(i);
            let emitted = is_emitted(@received_events, from, event);

            if !emitted {
                let from: felt252 = (*from).into();
                panic!("Event with matching data and keys was not emitted from {}", from);
            }

            i += 1;
        };
    }

    fn assert_not_emitted(ref self: EventSpy, events: @Array<(ContractAddress, T)>) {
        let mut i = 0;
        let received_events = self.get_events();

        while i < events.len() {
            let (from, event) = events.at(i);
            let emitted = is_emitted(@received_events, from, event);

            if emitted {
                let from: felt252 = (*from).into();
                panic!("Event with matching data and keys was emitted from {}", from);
            }

            i += 1;
        };
    }
}

fn is_emitted<T, impl TEvent: starknet::Event<T>, impl TDrop: Drop<T>>(
    self: @Events, expected_emitted_by: @ContractAddress, expected_event: @T
) -> bool {
    let mut expected_keys = array![];
    let mut expected_data = array![];
    expected_event.append_keys_and_data(ref expected_keys, ref expected_data);

    let mut i = 0;
    let mut is_emitted = false;
    while i < self.events.len() {
        let (from, event) = self.events.at(i);

        if from == expected_emitted_by
            && event.keys == @expected_keys
            && event.data == @expected_data {
            is_emitted = true;
            break;
        };

        i += 1;
    };
    return is_emitted;
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
