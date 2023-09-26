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

#[derive(Drop, Clone, Serde, PartialEq)]
enum Event {
    Named: NamedEvent,
    Unnamed: UnnamedEvent
}

#[generate_trait]
impl EventImpl of EventTrait {
    fn from(self: Event) -> ContractAddress {
        match self {
            Event::Named(NamedEvent { from, .. }) => from,
            Event::Unnamed(UnnamedEvent { from, .. }) => from,
        }
    }

    fn name(self: Event) -> Option<felt252> {
        match self {
            Event::Named(NamedEvent { name, .. }) => Option::Some(name),
            Event::Unnamed(_) => Option::None,
        }
    }

    fn keys(self: Event) -> Array<felt252> {
        match self {
            Event::Named(NamedEvent { keys, .. }) => keys,
            Event::Unnamed(UnnamedEvent { keys, .. }) => keys,
        }
    }

    fn data(self: Event) -> Array<felt252> {
        match self {
            Event::Named(NamedEvent { data, .. }) => data,
            Event::Unnamed(UnnamedEvent { data, .. }) => data,
        }
    }
}

#[derive(Drop, Clone, Serde, PartialEq)]
struct NamedEvent {
    from: ContractAddress,
    name: felt252,
    keys: Array<felt252>,
    data: Array<felt252>
}

#[derive(Drop, Clone, Serde, PartialEq)]
struct UnnamedEvent {
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

            let emitted = assert_if_emitted(ref self, events.at(i).clone());

            if !emitted {
                let name = match events.at(i) {
                    Event::Named(NamedEvent { name, ..}) => *name,
                    Event::Unnamed(_) => 'Unnamed'
                };

                panic(
                    array![
                        name,
                        'event with matching data and',
                        'keys was not emitted from',
                        (events.at(i).clone().from()).into()
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

        let emitted = match event.clone() {
            Event::Named(named_event) => {
                let mut cloned_event = named_event.clone();
                cloned_event.name = event_name_hash(cloned_event.name);
                Event::Named(cloned_event) == emitted_events.at(j).clone()
            },
            Event::Unnamed(unnamed_event) => Event::Unnamed(unnamed_event) == emitted_events.at(j).clone()
        };

        if emitted {
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
