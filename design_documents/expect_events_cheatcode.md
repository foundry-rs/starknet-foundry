# `expect_events` Cheatcode

## Context

Some contract functions can emit events. It is important to test if they were emitted properly.

## Goal

Propose a solution that will allow checking if events were emitted.

## Considered Solutions

### `expect_events` Cheatcode

Introduce a cheatcode with the signature:

```cairo
fn expect_events(events: Array<Event>)
```

That will define events which should be emitted in the next call. Other calls will not be affected.
If provided events will not be emitted it will panic with a detailed message.

`events` are the subsequence of all events emitted by the function, but you can also require function
to return exactly those (and not more) events with the `expect_exact_events` cheatcode. 

```cairo
fn expect_exact_events(events: Array<Event>)
```

It will panic if:
- not all defined events were emitted
- some other events where emitted

### To consider

1. Events are enum variants and are represented as a structs, so there are three ways of passing them to the cheatcode:
    1. We should allow passing only the enum variant
    2. We should allow passing them as a struct with all fields filled
    3. We should allow for both

### Usage example

```cairo
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

#[test]
fn check_emitted_event() {
    // ...
    let mut expected_events = ArrayTrait::new();
    	a.append(HelloEvent::StoredName);

	expect_events(expected_events);
    let res = contract.emit_store_name(...);  // if the event is not emitted it will panic

    let res = contract.emit_store_name(...);  // expect_events does not work here
    
    expect_exact_events(expected_events);
    let res = contract.emit_store_name(...);  // function has to emit exactly events defined in the array otherwise it will panic
    // ...
}
```
