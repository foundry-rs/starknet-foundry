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
    name: felt252,
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

trait EventAssertions {
    fn assert_emitted(ref self: EventSpy, events: @Array<Event>);
}

impl EventAssertionsImpl of EventAssertions {
    fn assert_emitted(ref self: EventSpy, events: @Array<Event>) {
        self.fetch_events();

        let mut i = 0;
        loop {
            if i >= events.len() {
                break;
            }

            let emitted = assert_if_emitted(ref self, copy_event(events.at(i)));

            if !emitted {
                panic(
                    array![
                        *events.at(i).name,
                        'event with matching data and',
                        'keys was not emitted from',
                        (*events.at(i).from).into()
                    ]
                );
            }

            i += 1;
        };
    }
}

fn assert_if_emitted(ref self: EventSpy, event: Event) -> bool {
    let emitted_events = @self.events;
    let mut j = 0;
    return loop {
        if j >= emitted_events.len() {
            break false;
        }

        if event_name_hash(event.name) == *self.events.at(j).name
          && event.from == *emitted_events.at(j).from
          && @event.keys == emitted_events.at(j).keys
          && @event.data == emitted_events.at(j).data {
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
            emitted_events_deleted_event.append(copy_event(emitted_events.at(k)));
        }
        k += 1;
    };
    self.events = emitted_events_deleted_event;
}

fn copy_event(event: @Event) -> Event {
    let from = *event.from;
    let name = *event.name;

    let mut keys = array![];
    let mut i = 0;
    loop {
        if i >= event.keys.len() {
            break;
        }

        keys.append(*event.keys.at(i));
        i += 1;
    };

    let mut data = array![];
    i = 0;
    loop {
        if i >= event.data.len() {
            break;
        }

        data.append(*event.data.at(i));
        i += 1;
    };

    Event { from, name, keys, data }
}
