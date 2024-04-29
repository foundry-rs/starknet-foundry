use core::array::ArrayTrait;
use core::option::OptionTrait;
use starknet::testing::cheatcode;
use starknet::ContractAddress;
use super::super::_cheatcode::handle_cheatcode;

/// Allows specifying which contracts you want to capture events from.
#[derive(Drop, Serde)]
enum SpyOn {
    All: (),
    One: ContractAddress,
    Multiple: Array<ContractAddress>
}

/// Creates `EventSpy` instance which spies on events emitted by contracts defined under the `spy_on` argument.
fn spy_events(spy_on: SpyOn) -> EventSpy {
    let mut inputs = array![];
    spy_on.serialize(ref inputs);
    let output = handle_cheatcode(cheatcode::<'spy_events'>(inputs.span()));

    EventSpy { _id: *output[0], events: array![] }
}

/// Raw event format (as seen via the RPC-API), can be used for asserting the emitted events.
#[derive(Drop, Clone, Serde)]
struct Event {
    keys: Array<felt252>,
    data: Array<felt252>
}


/// An event spy structure, along with the events collected so far in the test.
/// `events` are mutable and can be updated with `fetch_events`.
#[derive(Drop, Serde)]
struct EventSpy {
    _id: felt252,
    events: Array<(ContractAddress, Event)>,
}

trait EventFetcher {
    /// Allows to update the structs' events field, from the spied contracts
    fn fetch_events(ref self: EventSpy);
}

impl EventFetcherImpl of EventFetcher {
    fn fetch_events(ref self: EventSpy) {
        let mut output = handle_cheatcode(cheatcode::<'fetch_events'>(array![self._id].span()));
        let events = Serde::<Array<(ContractAddress, Event)>>::deserialize(ref output).unwrap();

        let mut i = 0;
        while i < events.len() {
            let (from, event) = events.at(i);
            self.events.append((*from, event.clone()));
            i += 1;
        }
    }
}

/// Allows to assert the expected events emission (or lack thereof), in the scope of the spy
trait EventAssertions<T, impl TEvent: starknet::Event<T>, impl TDrop: Drop<T>> {
    fn assert_emitted(ref self: EventSpy, events: @Array<(ContractAddress, T)>);
    fn assert_not_emitted(ref self: EventSpy, events: @Array<(ContractAddress, T)>);
}

impl EventAssertionsImpl<
    T, impl TEvent: starknet::Event<T>, impl TDrop: Drop<T>
> of EventAssertions<T> {
    fn assert_emitted(ref self: EventSpy, events: @Array<(ContractAddress, T)>) {
        self.fetch_events();

        let mut i = 0;
        while i < events.len() {
            let (from, event) = events.at(i);
            let emitted = is_emitted(ref self, from, event);

            if !emitted {
                let from: felt252 = (*from).into();
                panic!("Event with matching data and keys was not emitted from {}", from);
            }

            i += 1;
        };
    }

    fn assert_not_emitted(ref self: EventSpy, events: @Array<(ContractAddress, T)>) {
        self.fetch_events();

        let mut i = 0;
        while i < events.len() {
            let (from, event) = events.at(i);
            let emitted = is_emitted(ref self, from, event);

            if emitted {
                let from: felt252 = (*from).into();
                panic!("Event with matching data and keys was emitted from {}", from);
            }

            i += 1;
        };
    }
}

fn is_emitted<T, impl TEvent: starknet::Event<T>, impl TDrop: Drop<T>>(
    ref self: EventSpy, expected_from: @ContractAddress, expected_event: @T
) -> bool {
    let emitted_events = @self.events;

    let mut expected_keys = array![];
    let mut expected_data = array![];
    expected_event.append_keys_and_data(ref expected_keys, ref expected_data);

    let mut j = 0;
    let mut is_emitted = false;
    while j < emitted_events.len() {
        let (from, event) = emitted_events.at(j);

        if from == expected_from && event.keys == @expected_keys && event.data == @expected_data {
            remove_event(ref self, j);
            is_emitted = true;
            break;
        };

        j += 1;
    };
    return is_emitted;
}

fn remove_event(ref self: EventSpy, index: usize) {
    let emitted_events = @self.events;
    let mut emitted_events_deleted_event = array![];
    let mut k = 0;
    while k < emitted_events.len() {
        if k != index {
            let (from, event) = emitted_events.at(k);
            emitted_events_deleted_event.append((*from, event.clone()));
        }
        k += 1;
    };
    self.events = emitted_events_deleted_event;
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
