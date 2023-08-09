# Accessing emitted events

<!-- TOC -->
* [Accessing emitted events](#accessing-emitted-events)
  * [Context](#context)
  * [Goal](#goal)
  * [Considered Solutions](#considered-solutions)
  * [`expect_events` cheatcode](#expectevents-cheatcode)
    * [Usage example](#usage-example)
  * [`start_spy` cheatcode](#startspy-cheatcode)
    * [Usage example](#usage-example-1)
<!-- TOC -->

## Context

Some contract functions can emit events. It is important to test if they were emitted properly.

## Goal

Propose a solution that will allow checking if events were emitted.

## Considered Solutions

1. `expect_events` cheatcode 
2. `start_spy` cheatcode

## `expect_events` cheatcode

Introduce a cheatcode with the signature:

```cario
fn expect_events(events: Array<snforge_std::Event>)
```

where `snforge_std::Event` is defined as below

```cario
struct Event {
    name: felt252,
    keys: Array<felt252>,
    data: Array<felt252>
}
```

- `name` is a name of an event passed as a shortstring,
- `keys` are values under `#[key]`-marked fields of an event,
- `data` are all other values in emitted event.

`expect_event` cheatcode will define events which should be emitted in the next call. Other calls will not be affected.
If provided events will not be emitted it will panic with a detailed message.

`events` are the subsequence of all events emitted by the function, but you can also require function
to return exactly those (and not more) events with the `expect_exact_events` cheatcode. 

```cario
fn expect_exact_events(events: Array<snforge_std::Event>)
```

It will panic if:
- not all defined events were emitted
- some other events where emitted

### Usage example

```cario
#[starknet::interface]
trait IHelloEvent<TContractState> {
    fn emit_store_name(self: @TContractState, name: felt252);
}

mod HelloEvent {
    // ...
    
    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        StoredName: StoredName, 
    }

    #[derive(Drop, starknet::Event)]
    struct StoredName {
        #[key]
        user: ContractAddress,
        name: felt252
    }
    
    #[external(v0)]
    impl IHelloEventImpl of super::IHelloEvent<ContractState> {
        fn emit_store_name(self: @ContractState, name: felt252) {
            // ...
            self.emit(Event::StoredName(StoredName { user: get_caller_address(), name: name }));
        }
    }
}

use snforge_std::expect_events;
use snforge_std::Event;

#[test]
fn check_emitted_event() {
    // ...
	expect_events(array![Event { name: 'StoredName', keys: array![123], data: array![456] }]);
    let res = contract.emit_store_name(...);  // if the event is not emitted it will panic

    let res = contract.emit_store_name(...);  // expect_events does not work here
    
    expect_exact_events(array![Event { name: 'StoredName', keys: array![123], data: array![456] }]);
    let res = contract.emit_store_name(...);  // function has to emit exactly those events defined in the array otherwise it will panic
    // ...
}
```

## `start_spy` cheatcode

Another idea is to give users the highest possible customization. There would be a handler for extracting events emitted
after the line it was created. Users would have to assert events manually (which could be more complex than the previous
solution but would add more flexibility).

Introduce `start_spy` cheatcode with the signature:

```cario
fn start_spy() -> snforge_std::EventSpy
```

where `snforge_std::EventSpy` would allow for accessing events emitted after its creation.
`EventSpy` would be defined as follows.

```cario
struct EventSpy {
    // mapping between contract address and its events
    events: Felt252Dict<Array<snforge_std::Event>>,
}

trait EventFetcher {
    fn fetch_events(self: EventSpy);
}

impl EventFetcherImpl of EventFetcher {
    fn fetch_events(self: EventSpy) {
        // ...
    }
}
```

Users will be responsible for calling `fetch_events` to load emitted events to the `events` property.

It would be important to somehow end spying (if users want to check events only in some places), so we will introduce

```cario
fn stop_spy()
```

cheatcode which will stop events collection. After `stop_spy` usage, handler will not be modifier (no more events will be added).

### Usage example

```cario
// Contract is the same as in the previous example

use snforge_std::start_spy;
use snforge_std::stop_spy;
use snforge_std::EventSpy;

#[test]
fn check_emitted_event() {
    // ...
	let mut spy = start_spy();  // all events emitted after this line will be saved under the `spy` variable
    let res = contract.emit_store_name(...);
    
    spy.fetch_events();
    assert!(spy.events.get(contract_address).len() == 1, 'There should be one event');

    let res = contract.emit_store_name(...);
    let res = contract.emit_store_name(...);
    
    assert!(spy.events.get(contract_address).len() == 1, 'There should be one event');
    
    spy.fetch_events();
    let contract_events = spy.events.get(contract_address);
    assert!(contract_events.len() == 3, 'There should be three events');
    
    let mut i = 0;
    loop {
        if i >= contract_events.len() {
            break;
        }
        assert!(contract_events.at(i).name == 'StoredName', 'Unexpected event name');
        i += 1;
    }
    
    stop_spy();  // no more events will be added to the handler
    
    let res = contract.emit_store_name(...);
    
    spy.fetch_events();
    assert!(spy.events.get(contract_address).len() == 3, 'There should be three events');
    // ...
}
```


