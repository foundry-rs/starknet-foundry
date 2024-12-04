# Testing events
Examples are based on the following `SpyEventsChecker` contract implementation:

```rust
{{#include ../../listings/testing_events/src/contract.cairo}}
```

## Asserting emission with `assert_emitted` method

This is the simpler way, in which you don't have to fetch the events explicitly.
See the below code for reference:

```rust
{{#include ../../listings/testing_events/tests/assert_emitted.cairo}}
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
{{#include ../../listings/testing_events/tests/assert_manually.cairo}}
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
{{#include ../../listings/testing_events/tests/filter.cairo}}
```

`events_from_first_address` has events emitted by the first contract only.
Similarly, `events_from_second_address` has events emitted by the second contract.

## Asserting Events Emitted With `emit_event_syscall`

Events emitted with `emit_event_syscall` could have nonstandard (not defined anywhere) keys and data.
They can also be asserted with `spy.assert_emitted` method.

Let's extend our `SpyEventsChecker` with `emit_event_with_syscall` method:

```rust
{{#include ../../listings/testing_events/src/syscall_dummy.cairo}}
```

And add a test for it:

```rust
{{#include ../../listings/testing_events/tests/syscall.cairo}}
```

Using `Event` struct from the `snforge_std` library we can easily assert nonstandard events.
This also allows for testing the events you don't have the code of, or you don't want to import those.
