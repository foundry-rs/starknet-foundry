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
use snforge_std::{declare, ContractClassTrait, spy_events, SpyOn, EventSpy,
    EventAssertions};

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

    let mut spy = spy_events(SpyOn::One(contract_address));

    dispatcher.emit_one_event(123);

    spy.assert_emitted(@array![
        (
            contract_address,
            SpyEventsChecker::Event::FirstEvent(
                SpyEventsChecker::FirstEvent { some_data: 123 }
            )
        )
    ]);
    assert(spy.events.len() == 0, 'There should be no events');
}
```

Let's go through the code:

1. After the contract is called, we don't have to call `fetch_events` on the spy (it is done inside
  the `assert_emitted` method).
2. `assert_emitted` takes the array snapshot of tuples `(ContractAddress, event)` we expect were emitted.
3. After the assertion, found events are removed from the spy. It stays clean and ready for the next events.

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
You can also use the `event` field directly and assert data selectively, if you don't want to assert the whole thing.
This however, requires you to fetch the events **manually**.
 
```rust
use snforge_std::{declare, ContractClassTrait, spy_events, SpyOn, EventSpy, EventFetcher, Event};

#[starknet::interface]
trait ISpyEventsChecker<TContractState> {
    fn emit_one_event(ref self: TContractState, some_data: felt252);
}

#[test]
fn test_complex_assertions() {
    let contract = declare("SpyEventsChecker").unwrap();
    let (contract_address, _) = contract.deploy(array![]).unwrap();
    let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

    let mut spy = spy_events(SpyOn::One(contract_address)); // Ad 1.

    dispatcher.emit_one_event(123);

    spy.fetch_events();  // Ad 2.

    assert(spy.events.len() == 1, 'There should be one event');

    let (from, event) = spy.events.at(0); // Ad 3.
    assert(from == @contract_address, 'Emitted from wrong address');
    assert(event.keys.len() == 1, 'There should be one key');
    assert(event.keys.at(0) == @selector!("FirstEvent"), 'Wrong event name'); // Ad 4.
    assert(event.data.len() == 1, 'There should be one data');

    dispatcher.emit_one_event(123);
    assert(spy.events.len() == 1, 'There should be one event'); // Ad 5. - Still one event

    spy.fetch_events();
    assert(spy.events.len() == 2, 'There should be two events');
}
```

Let's go through important parts of the provided code:

1. After contract deployment we created the spy with `spy_events` cheatcode.
  From this moment all events emitted by the `SpyEventsChecker` contract will be spied.
2. We have to call `fetch_events` method on the created spy to load emitted events into it.
3. When events are fetched they are loaded into the `events` property of our spy, and we can assert them.
4. If the event is emitted by calling `self.emit` method, its hashed name is saved under the `keys.at(0)`
(this way Starknet handles events)
5. It is worth noting that when we call the method which emits an event, `spy` is not updated immediately.

> 📝 **Note**
> To assert the `name` property we have to hash a string with the `selector!` macro.


## Splitting Events Between Multiple Spies

Sometimes it is easier to split events between multiple spies.
For example - one spy for ERC20 contract, and one for your own contracts. 
Let's do it.

```rust
use snforge_std::{declare, ContractClassTrait, spy_events, SpyOn, EventSpy, EventAssertions};

use SpyEventsChecker;

#[starknet::interface]
trait ISpyEventsChecker<TContractState> {
    fn emit_one_event(ref self: TContractState, some_data: felt252);
}

#[test]
fn test_simple_assertions() {
    let contract = declare("SpyEventsChecker").unwrap();
    let (first_address, _) = contract.deploy(array![]).unwrap();
    let (second_address, _) = contract.deploy(array![]).unwrap();
    let (third_address, _) = contract.deploy(array![]).unwrap();

    let first_dispatcher = ISpyEventsCheckerDispatcher { first_address };
    let second_dispatcher = ISpyEventsCheckerDispatcher { second_address };
    let third_dispatcher = ISpyEventsCheckerDispatcher { third_address };

    let mut spy_one = spy_events(SpyOn::One(first_address));
    let mut spy_two = spy_events(SpyOn::Multiple(array![second_address, third_address]));

    first_dispatcher.emit_one_event(123);
    second_dispatcher.emit_one_event(234);
    third_dispatcher.emit_one_event(345);

    spy_one.assert_emitted(@array![
        (
            first_address,
            SpyEventsChecker::Event::FirstEvent(
                SpyEventsChecker::FirstEvent { some_data: 123 }
            )
        )
    ]);
    spy_two.assert_emitted(@array![
        (
            second_address,
            SpyEventsChecker::Event::FirstEvent(
                SpyEventsChecker::FirstEvent { some_data: 234 }
            )
        ),
        (
            third_address,
            SpyEventsChecker::Event::FirstEvent(
                SpyEventsChecker::FirstEvent { some_data: 345 }
            )
        )
    ]);
}
```

The first spy gets events emitted by the first contract only. Second one gets events emitted by the rest.

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
use snforge_std::{ declare, ContractClassTrait, spy_events, EventSpy, EventFetcher,
    EventAssertions, Event, SpyOn };

#[starknet::interface]
trait ISpyEventsChecker<TContractState> {
    fn emit_event_syscall(ref self: TContractState, some_key: felt252, some_data: felt252);
}

#[test]
fn test_simple_assertions() {
    let contract = declare("SpyEventsChecker").unwrap();
    let (contract_address, _) = contract.deploy(array![]).unwrap();
    let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

    let mut spy = spy_events(SpyOn::One(contract_address));
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


> ⚠️ **Warning**
>
> Spying on the same contract with multiple spies can result in unexpected behavior — avoid it if possible.

