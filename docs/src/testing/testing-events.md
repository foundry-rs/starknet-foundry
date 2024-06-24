# Testing events
Examples are based on the following `SpyEventsChecker` contract implementation:

```rust
#[starknet::contract]
mod SpyEventsChecker {
    // ...

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        FirstEvent: FirstEvent
    }

    #[derive(Drop, starknet::Event)]
    struct FirstEvent {
        some_data: felt252
    }

    // ...
}
```

## Asserting emission with `assert_emitted` method

This is the simpler way, in which you don't have to fetch the events explicitly.
See the below code for reference:

```rust
use snforge_std::{declare, ContractClassTrait, spy_events, EventSpy, EventSpyTrait,
      EventSpyAssertionsTrait};

use SpyEventsChecker;

#[starknet::interface]
trait ISpyEventsChecker<TContractState> {
    fn emit_one_event(ref self: TContractState, some_data: felt252);
}

#[test]
fn test_simple_assertions() {
    let contract = declare("SpyEventsChecker").unwrap();
    let (contract_address, _) = contract.deploy(array![]).unwrap();
    let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

    let mut spy = spy_events();  // Ad. 1

    dispatcher.emit_one_event(123);

    spy.assert_emitted(@array![  // Ad. 2
        (
            contract_address,
            SpyEventsChecker::Event::FirstEvent(
                SpyEventsChecker::FirstEvent { some_data: 123 }
            )
        )
    ]);
}
```

Let's go through the code:

1. After contract deployment, we created the spy using `spy_events` cheatcode. From this moment all emitted events 
will be spied.
2. Asserting is done using the `assert_emitted` method. It takes an array snapshot of `(ContractAddress, event)`
tuples we expect that were emitted.

> 📝 **Note**
> We can pass events defined in the contract and construct them like in the `self.emit` method!


## Asserting lack of event emission with `assert_not_emitted`

In cases where you want to test an event was *not* emitted, use the `assert_not_emitted` function.
It works similarly as `assert_emitted` with the only difference that it panics if an event was emitted during the execution.

Given the example above, we can check that a different `FirstEvent` was not emitted:

```rust
spy.assert_not_emitted(@array![
    (
        contract_address,
        SpyEventsChecker::Event::FirstEvent(
            SpyEventsChecker::FirstEvent { some_data: 456 }
        )
    )
]);
```

Note that both the event name and event data are checked. 
If a function emitted an event with the same name but a different payload, the `assert_not_emitted` function will pass.

## Asserting the events manually
If you wish to assert the data manually, you can do that on the `Events` structure. 
Simply call `get_events()` on your `EventSpy` and access `events`  field on the returned `Events` value.
Then, you can access the events and assert data by yourself.

```rust
use snforge_std::{declare, ContractClassTrait, spy_events, EventSpyTrait, EventsAssertionsTrait,
      Event};

#[starknet::interface]
trait ISpyEventsChecker<TContractState> {
    fn emit_one_event(ref self: TContractState, some_data: felt252);
}

#[test]
fn test_complex_assertions() {
    let contract = declare("SpyEventsChecker").unwrap();
    let (contract_address, _) = contract.deploy(array![]).unwrap();
    let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

    let mut spy = spy_events(); // Ad 1.

    dispatcher.emit_one_event(123);

    let events = spy.get_events();  // Ad 2.

    assert(events.events.len() == 1, 'There should be one event');

    let (from, event) = events.events.at(0); // Ad 3.
    assert(from == @contract_address, 'Emitted from wrong address');
    assert(event.keys.len() == 1, 'There should be one key');
    assert(event.keys.at(0) == @selector!("FirstEvent"), 'Wrong event name'); // Ad 4.
    assert(event.data.len() == 1, 'There should be one data');
}
```

Let's go through important parts of the provided code:

1. After contract deployment we created the spy with `spy_events` cheatcode.
From this moment all events emitted by the `SpyEventsChecker` contract will be spied.
2. We have to call `get_events` method on the created spy to fetch our events and get the `Events` structure.
3. To get our particular event, we need to access the `events` property and get the event under an index.
Since `events` is an array holding a tuple of `ContractAddress` and `Event`, we unpack it using `let (from, event)`.
4. If the event is emitted by calling `self.emit` method, its hashed name is saved under the `keys.at(0)`
(this way Starknet handles events)

> 📝 **Note**
> To assert the `name` property we have to hash a string with the `selector!` macro.


## Filtering Events

Sometimes, when you assert the events manually, you might not want to get all the events, but only ones from
a particular address. You can address that by using the method `emitted_by` on the `Events` structure. 

```rust
use snforge_std::{declare, ContractClassTrait, spy_events, EventSpyTrait,
      EventsAssertionsTrait, EventsFilterTrait};

use SpyEventsChecker;

#[starknet::interface]
trait ISpyEventsChecker<TContractState> {
    fn emit_one_event(ref self: TContractState, some_data: felt252);
}

#[test]
fn test_assertions_with_filtering() {
    let contract = declare("SpyEventsChecker").unwrap();
    let (first_address, _) = contract.deploy(array![]).unwrap();
    let (second_address, _) = contract.deploy(array![]).unwrap();

    let first_dispatcher = ISpyEventsCheckerDispatcher { contract_address: first_address };
    let second_dispatcher = ISpyEventsCheckerDispatcher { contract_address: second_address };

    let mut spy = spy_events();

    first_dispatcher.emit_one_event(123);
    second_dispatcher.emit_one_event(234);
    second_dispatcher.emit_one_event(345);

    let events_from_first_address = spy.get_events().emitted_by(first_address);
    let events_from_second_address = spy.get_events().emitted_by(second_address);

    let (from_first, event_from_first) = events_from_first_address.events.at(0);
    assert(from_first == @first_address, 'Emitted from wrong address');
    assert(event_from_first.data.at(0) == @123.into(), 'Data should be 123');

    let (from_second_one, event_from_second_one) = events_from_second_address.events.at(0);
    assert(from_second_one == @second_address, 'Emitted from wrong address');
    assert(event_from_second_one.data.at(0) == @234.into(), 'Data should be 234');

    let (from_second_two, event_from_second_two) = events_from_second_address.events.at(1);
    assert(from_second_two == @second_address, 'Emitted from wrong address');
    assert(event_from_second_two.data.at(0) == @345.into(), 'Data should be 345');
}
```

`events_from_first_address` has events emitted by the first contract only.
Similarly, `events_from_second_address` has events emitted by the second contract.

## Asserting Events Emitted With `emit_event_syscall`

Events emitted with `emit_event_syscall` could have nonstandard (not defined anywhere) keys and data.
They can also be asserted with `spy.assert_emitted` method.

Let's consider such a method in the `SpyEventsChecker` contract.

```rust
fn emit_event_syscall(ref self: ContractState, some_key: felt252, some_data: felt252) {
    starknet::emit_event_syscall(array![some_key].span(), array![some_data].span()).unwrap_syscall();
}
```

And the test.

```rust
use snforge_std::{ declare, ContractClassTrait, spy_events, EventSpyTrait,
    EventsAssertionsTrait, Event };

#[starknet::interface]
trait ISpyEventsChecker<TContractState> {
    fn emit_event_syscall(ref self: TContractState, some_key: felt252, some_data: felt252);
}

#[test]
fn test_nonstandard_events() {
    let contract = declare("SpyEventsChecker").unwrap();
    let (contract_address, _) = contract.deploy(array![]).unwrap();
    let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

    let mut spy = spy_events();
    dispatcher.emit_event_syscall(123, 456);

    spy.assert_emitted(@array![
        (
            contract_address,
            Event { keys: array![123], data: array![456] }
        )
    ]);
}
```

Using `Event` struct from the `snforge_std` library we can easily assert nonstandard events.
This also allows for testing the events you don't have the code of, or you don't want to import those.
