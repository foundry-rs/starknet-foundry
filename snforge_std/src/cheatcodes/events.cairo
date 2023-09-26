use array::ArrayTrait;
use array::SpanTrait;
use clone::Clone;
use serde::Serde;
use option::OptionTrait;
use traits::Into;

use starknet::testing::cheatcode;
use starknet::ContractAddress;

#[derive(Drop, Serde)]
enum SpyOn {
    All: (),
    One: ContractAddress,
    Multiple: Array<ContractAddress>
}

fn spy_events(spy_on: SpyOn) -> EventSpy {
    let mut inputs = array![];
    spy_on.serialize(ref inputs);
    let output = cheatcode::<'spy_events'>(inputs.span());

    EventSpy { _id: *output[0], events: array![] }
}

fn event_name_hash(name: felt252) -> felt252 {
    let mut output = cheatcode::<'event_name_hash'>(array![name].span());
    *output[0]
}

#[derive(Drop, Clone, Serde)]
struct Event {
    from: ContractAddress,
    keys: Array<felt252>,
    data: Array<felt252>
}

#[derive(Drop, Clone, Serde)]
struct EventSpy {
    _id: felt252,
    events: Array<Event>,
}

trait EventFetcher {
    fn fetch_events(ref self: EventSpy);
}

impl EventFetcherImpl of EventFetcher {
    fn fetch_events(ref self: EventSpy) {
        let mut output = cheatcode::<'fetch_events'>(array![self._id].span());
        let events = Serde::<Array<Event>>::deserialize(ref output).unwrap();

        let mut i = 0;
        loop {
            if i >= events.len() {
                break;
            }
            self.events.append(events.at(i).clone());
            i += 1;
        }
    }
}

trait EventAssertions<T, impl TEvent: starknet::Event<T>, impl TDrop: Drop<T>> {
    fn assert_emitted(ref self: EventSpy, events: @Array<T>);
}

impl EventAssertionsImpl<
    T, impl TEvent: starknet::Event<T>, impl TDrop: Drop<T>
> of EventAssertions<T> {
    fn assert_emitted(ref self: EventSpy, events: @Array<T>) {
        self.fetch_events();

        let mut i = 0;
        loop {
            if i >= events.len() {
                break;
            }

            let emitted = assert_if_emitted(ref self, events.at(i));

            if !emitted {
                panic(array!['Event with matching data and', 'keys was not emitted',]);
            }

            i += 1;
        };
    }
}

fn assert_if_emitted<T, impl TEvent: starknet::Event<T>, impl TDrop: Drop<T>>(
    ref self: EventSpy, event: @T
) -> bool {
    let emitted_events = @self.events;
    let mut j = 0;
    return loop {
        if j >= emitted_events.len() {
            break false;
        }

        let mut keys = array![];
        let mut data = array![];
        event.append_keys_and_data(ref keys, ref data);

        if keys == emitted_events.at(j).keys.clone() && data == emitted_events.at(j).data.clone() {
            remove_event(ref self, j);
            break true;
        }

        j += 1;
    };
}

fn remove_event(ref self: EventSpy, index: usize) {
    let emitted_events = @self.events;
    let mut emitted_events_deleted_event = array![];
    let mut k = 0;
    loop {
        if k >= emitted_events.len() {
            break;
        }

        if k != index {
            emitted_events_deleted_event.append(emitted_events.at(k).clone());
        }
        k += 1;
    };
    self.events = emitted_events_deleted_event;
}
