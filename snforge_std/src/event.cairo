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
    cheatcode::<'spy_events'>(inputs.span());

    EventSpy { events: array![] }
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
    events: Array<Event>,
}

trait EventFetcher {
    fn fetch_events(ref self: EventSpy);
}

impl EventFetcherImpl of EventFetcher {
    fn fetch_events(ref self: EventSpy) {
        let mut output = cheatcode::<'fetch_events'>(array![].span());
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
        let all_found = loop {
            if i >= events.len() {
                break true;
            }

            let mut emitted_events = @self.events;
            let mut j = 0;
            let found = loop {
                if j >= emitted_events.len() {
                    break false;
                }

                if event_name_hash(*events.at(i).name) == *emitted_events.at(j).name
                    && events.at(i).from == emitted_events.at(j).from
                {
                    if events.at(i).keys != emitted_events.at(j).keys
                        || events.at(i).data != emitted_events.at(j).data
                    {
                        panic(array![*events.at(i).name, 'event was emitted from', (*events.at(i).from).into(),
                            'but keys or data are different'
                        ]);
                    }

                    let mut emitted_events_deleted_event = array![];
                    let mut k = 0;
                    loop {
                        if k >= emitted_events.len() {
                            break;
                        }

                        if k != j {
                            emitted_events_deleted_event.append(copy_event(emitted_events.at(k)));
                        }
                        k += 1;
                    };
                    self.events = emitted_events_deleted_event;

                    break true;
                }

                j += 1;
            };

            if !found {
                panic(array![*events.at(i).name, 'event was not emitted from', (*events.at(i).from).into()]);
            }

            i += 1;
        };
    }
}

fn copy_event(event: @Event) -> Event {
    let from = *event.from;
    let name = *event.name;

    let mut keys = array![];
    let mut i = 0;
    loop {
        if i >= event.keys.len() { break; }

        keys.append(*event.keys.at(i));
        i += 1;
    };

    let mut data = array![];
    i = 0;
    loop {
        if i >= event.data.len() { break; }

        data.append(*event.data.at(i));
        i += 1;
    };

    Event { from, name, keys, data }
}

// Copied from corelib until it will be included in the release
impl ArrayPartialEq<T, impl PartialEqImpl: PartialEq<T>> of PartialEq<Array<T>> {
    fn eq(lhs: @Array<T>, rhs: @Array<T>) -> bool {
        lhs.span() == rhs.span()
    }
    fn ne(lhs: @Array<T>, rhs: @Array<T>) -> bool {
        !(lhs == rhs)
    }
}

impl SpanPartialEq<T, impl PartialEqImpl: PartialEq<T>> of PartialEq<Span<T>> {
    fn eq(lhs: @Span<T>, rhs: @Span<T>) -> bool {
        if (*lhs).len() != (*rhs).len() {
            return false;
        }
        let mut lhs_span = *lhs;
        let mut rhs_span = *rhs;
        loop {
            match lhs_span.pop_front() {
                Option::Some(lhs_v) => {
                    if lhs_v != rhs_span.pop_front().unwrap() {
                        break false;
                    }
                },
                Option::None => {
                    break true;
                },
            };
        }
    }
    fn ne(lhs: @Span<T>, rhs: @Span<T>) -> bool {
        !(lhs == rhs)
    }
}
