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
    keys: Array<felt252>,
    data: Array<felt252>
}

#[derive(Drop, Serde)]
struct EventSpy {
    _id: felt252,
    events: Array<(ContractAddress, Event)>,
}

trait EventFetcher {
    fn fetch_events(ref self: EventSpy);
}

impl EventFetcherImpl of EventFetcher {
    fn fetch_events(ref self: EventSpy) {
        let mut output = cheatcode::<'fetch_events'>(array![self._id].span());
        let events = Serde::<Array<(ContractAddress, Event)>>::deserialize(ref output).unwrap();

        let mut i = 0;
        loop {
            if i >= events.len() {
                break;
            }
            let (from, event) = events.at(i);
            self.events.append((*from, event.clone()));
            i += 1;
        }
    }
}

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
        loop {
            if i >= events.len() {
                break;
            }

            let (from, event) = events.at(i);
            let emitted = is_emitted(ref self, from, event);

            if !emitted {
                panic(
                    array![
                        'Event with matching data and', 'keys was not emitted from', (*from).into()
                    ]
                );
            }

            i += 1;
        };
    }

    fn assert_not_emitted(ref self: EventSpy, events: @Array<(ContractAddress, T)>) {
        self.fetch_events();

        let mut i = 0;
        loop {
            if i >= events.len() {
                break;
            }

            let (from, event) = events.at(i);
            let emitted = is_emitted(ref self, from, event);

            if emitted {
                panic(
                    array!['Event with matching data and', 'keys was emitted from', (*from).into()]
                );
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
    return loop {
        if j >= emitted_events.len() {
            break false;
        }
        let (from, event) = emitted_events.at(j);

        if from == expected_from && event.keys == @expected_keys && event.data == @expected_data {
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
            let (from, event) = emitted_events.at(k);
            emitted_events_deleted_event.append((*from, event.clone()));
        }
        k += 1;
    };
    self.events = emitted_events_deleted_event;
}

impl EventTraitImpl of starknet::Event<Event> {
    fn append_keys_and_data(self: @Event, ref keys: Array<felt252>, ref data: Array<felt252>) {
        array_extend(ref keys, self.keys);
        array_extend(ref data, self.data);
    }

    fn deserialize(ref keys: Span<felt252>, ref data: Span<felt252>) -> Option<Event> {
        Option::None
    }
}

fn array_extend<T, impl TCopy: Copy<T>, impl TDrop: Drop<T>>(
    ref array: Array<T>, other: @Array<T>
) {
    let mut i = 0;
    loop {
        if i == other.len() {
            break;
        }
        array.append(*other.at(i));

        i += 1;
    };
}
