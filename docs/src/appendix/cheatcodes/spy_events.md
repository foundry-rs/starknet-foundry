# `spy_events`

> `fn spy_events(spy_on: SpyOn) -> EventSpy`

Creates `EventSpy` instance which spies on events emitted by contracts defined
under the `spy_on` argument.

```rust
struct EventSpy {
    events: Array<Event>,
}

struct Event {
    from: ContractAddress,
    name: felt252,
    keys: Array<felt252>,
    data: Array<felt252>
}

enum SpyOn {
    All: (),
    One: ContractAddress,
    Multiple: Array<ContractAddress>
}
```

> ⚠️ **Warning**
>
> Spying on the same contract with multiple spies can result in unexpected behavior — avoid it if possible.

`EventSpy` implements `EventFetcher` and `EventAssertions` traits.

```rust
trait EventFetcher {
    fn fetch_events(ref self: EventSpy);
}

trait EventAssertions {
    fn assert_emitted(ref self: EventSpy, events: @Array<Event>);
}
```

## There are two ways of interaction with emitted events

### Complex one in which user asserts manually

```rust
use snforge_std::{declare, ContractClassTrait, spy_events, SpyOn, EventSpy, EventFetcher
    event_name_hash, Event};
    
#[starknet::interface]
trait ISpyEventsChecker<TContractState> {
    fn emit_one_event(ref self: TContractState, some_data: felt252);
}

#[test]
fn test_complex_assertions() {
    let contract = declare('SpyEventsChecker');
    let contract_address = contract.deploy(array![]).unwrap();
    let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

    let mut spy = spy_events(SpyOn::One(contract_address));

    dispatcher.emit_one_event(123);

    spy.fetch_events();
    
    assert(spy.events.len() == 1, 'There should be one event');
    assert(spy.events.at(0).name == @event_name_hash('FirstEvent'), 'Wrong event name');
    assert(spy.events.at(0).keys.len() == 0, 'There should be no keys');
    assert(spy.events.at(0).data.len() == 1, 'There should be one data');
    
    dispatcher.emit_one_event(123);
    assert(spy.events.len() == 1, 'There should be one event');

    spy.fetch_events();
    assert(spy.events.len() == 2, 'There should be two events');
}
```

Let's go through important parts of the provided code:

- After contract deployment we created the spy with `spy_events` cheatcode.
  From this moment all events emitted by the `SpyEventsChecker` contract will be spied.
- We have to call `fetch_events` method on the created spy to load emitted events into it.
- When events are fetched they are loaded into the `events` property of our spy and we can assert them.

> 📝 **Note**
> To assert the `name` property we have to hash a shortstring with the `event_name_hash` cheatcode.
> 
> `fn event_name_hash(name: felt252) -> felt252`

- It is worth noting that when we call the method which emits an event, `spy` is not updated immediately.
  (See the last 5 lines)

### Simple one in which user asserts with `assert_emitted` method

```rust
use snforge_std::{declare, ContractClassTrait, spy_events, SpyOn, EventSpy, EventFetcher
    event_name_hash, Event, EventAssertions};
    
#[starknet::interface]
trait ISpyEventsChecker<TContractState> {
    fn emit_one_event(ref self: TContractState, some_data: felt252);
}

#[test]
fn test_simple_assertions() {
    let contract = declare('SpyEventsChecker');
    let contract_address = contract.deploy(array![]).unwrap();
    let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

    let mut spy = spy_events(SpyOn::One(contract_address));

    dispatcher.emit_one_event(123);

    spy.assert_emitted(@array![
        Event { from: contract_address, name: 'FirstEvent', keys: array![], data: array![123] }
    ]);
    assert(spy.events.len() == 0, 'There should be no events');
}
```

The flow is much simpler (thanks to `EventAssertions` trait). Let's go through it as previously:

- When contract is called we don't have to call `fetch_events` on the spy (it is done inside
  the `assert_emitted` method).
- `assert_emitted` takes the array snapshot of events we expect were emitted.

> 📝 **Note**
> This time we just pass the shortstring to the `name` property (there is no hashing!).

- After the assertion, found events are removed from the spy. It stays clean and ready for the next events.

## Splitting events between multiple spies

Sometimes it is easier to split events between multiple spies. Let's do it.

```rust
use snforge_std::{declare, ContractClassTrait, spy_events, SpyOn, EventSpy, EventFetcher
    event_name_hash, Event, EventAssertions};
    
#[starknet::interface]
trait ISpyEventsChecker<TContractState> {
    fn emit_one_event(ref self: TContractState, some_data: felt252);
}

#[test]
fn test_simple_assertions() {
    let contract = declare('SpyEventsChecker');
    let first_address = contract.deploy(array![]).unwrap();
    let second_address = contract.deploy(array![]).unwrap();
    let third_address = contract.deploy(array![]).unwrap();
    
    let first_dispatcher = ISpyEventsCheckerDispatcher { first_address };
    let second_dispatcher = ISpyEventsCheckerDispatcher { second_address };
    let third_dispatcher = ISpyEventsCheckerDispatcher { third_address };

    let mut spy_one = spy_events(SpyOn::One(first_address));
    let mut spy_two = spy_events(SpyOn::Multiple(array![second_address, third_address]));

    first_dispatcher.emit_one_event(123);
    second_dispatcher.emit_one_event(234);
    third_dispatcher.emit_one_event(345);

    spy_one.assert_emitted(@array![
        Event { from: first_address, name: 'FirstEvent', keys: array![], data: array![123] }
    ]);
    spy_two.assert_emitted(@array![
        Event { from: second_address, name: 'FirstEvent', keys: array![], data: array![234] },
        Event { from: third_address, name: 'FirstEvent', keys: array![], data: array![345] }
    ]);
}
```

The first spy gets events emitted by the first contract only. Second one gets events emitted by the rest.
