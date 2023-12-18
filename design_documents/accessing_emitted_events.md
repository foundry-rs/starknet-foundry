# Accessing emitted events

<!-- TOC -->
* [Accessing emitted events](#accessing-emitted-events)
  * [Context](#context)
  * [Goal](#goal)
  * [Considered Solutions](#considered-solutions)
  * [`expect_events` cheatcode](#expectevents-cheatcode)
    * [Usage example](#usage-example)
  * [`spy_events` cheatcode](#spyevents-cheatcode)
    * [Propositions to consider](#propositions-to-consider)
    * [Usage example](#usage-example-1)
<!-- TOC -->

## Context

Some contract functions can emit events. It is important to test if they were emitted properly.

## Goal

Propose a solution that will allow checking if events were emitted.

## Considered Solutions

1. `expect_events` cheatcode 
2. `spy_events` cheatcode

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

`events` are the subset of all events emitted by the function, but you can also require function
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
    
    #[abi(embed_v0)]
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

## `spy_events` cheatcode

Another idea is to give users the highest possible customization. There would be a handler for extracting events emitted
after the line it was created. Users would have to assert events manually (which could be more complex than the previous
solution but would add more flexibility).

Introduce `spy_events` cheatcode with the signature:

```cario
fn spy_events() -> snforge_std::EventSpy
```

where `snforge_std::EventSpy` would allow for accessing events emitted after its creation.
`EventSpy` would be defined as follows.

```cario
struct EventSpy {
    events: Array<snforge_std::Event>,
}

struct Event {
    from: ContractAddress,
    name: felt252,
    keys: Array<felt252>,
    data: Array<felt252>
}

trait EventFetcher {
    fn fetch_events(ref self: EventSpy);
}
```

Users will be responsible for calling `fetch_events` to load emitted events to the `events` property.

There will also be `assert_emitted` method available on the `EventSpy` struct.

```cairo
trait EventAssertions {
    fn assert_emitted(ref self: EventSpy, events: Array<snforge_std::Event>);
}
```

It is designed to enable more simplified flow:
- `fetch_events` will be called internally, so there will always be the newest events,
- checked events will be removed from the `events` property.

### Propositions to consider

- `TargetedSpy` - regular spy with the target parameter for creation, which would be an enumeration of:
  ```cairo
  enum SpyOn {
      All,
      One(ContractAddress),
      Multiple(Array<ContractAddress>>)
  }
  ```
- Two different classes for Events - one for incoming events and the second one for events created by users. 
  It would clarify the confusion when `name` field is hashed when it comes from `EventSpy`.


### Usage example

```cario
// Contract is the same as in the previous example

use snforge_std::spy_events;
use snforge_std::EventSpy;
use snforge_std::EventFetcher;
use snforge_std::EventAsserions;
use snforge_std::event_name_hash;

#[test]
fn check_emitted_event_simple() {
    // ...
	let mut spy = spy_events();  // all events emitted after this line will be saved under the `spy` variable
    let res = contract.emit_store_name(...);
    
    // after this line there will be no events under the `spy.events`
    spy.assert_emitted(array![Event {from: ..., name: 'StoredName', ...}]);

    let res = contract.emit_store_name(...);
    let res = contract.emit_store_name(...);
    
    spy.assert_emitted(
        array![
            Event {from: ..., name: 'StoredName', ...},
            Event {from: ..., name: 'StoredName', ...}
        ]
    );
    
    assert(spy.events.len() == 0, 'All events should be consumed');
    // ...
}

#[test]
fn check_emitted_event_complex() {
    // ...
	let mut spy = spy_events();  // all events emitted after this line will be saved under the `spy` variable
    let res = contract.emit_store_name(...);
    
    spy.fetch_events();
    
    // users can assert events on their own
    assert(spy.events.len() == 1, 'There should be one event');
    assert(spy.events.at(0).name == event_name_hash('StoredName'), 'Wrong event name');
    
    let data = array![...];
    assert(spy.events.at(0).data == data, 'Wrong data');

    let res = contract.emit_store_name(...);
    let res = contract.emit_store_name(...);
    
    // events will not be present before fetching
    assert!(spy.events.len() == 1, 'There should be one event');
    
    spy.fetch_events();
    assert(spy.events.len() == 3, 'There should be three events');
    // ...
```


