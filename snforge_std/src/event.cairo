use array::ArrayTrait;
use array::SpanTrait;
use clone::Clone;
use serde::Serde;
use option::OptionTrait;
use traits::Into;

use starknet::testing::cheatcode;
use starknet::ContractAddress;

fn spy_events() -> EventSpy {
    cheatcode::<'spy_events'>(array![].span());
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
