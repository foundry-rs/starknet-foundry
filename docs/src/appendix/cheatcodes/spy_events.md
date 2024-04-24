# `spy_events`

> `fn spy_events(spy_on: SpyOn) -> EventSpy`

Creates `EventSpy` instance which spies on events emitted by contracts defined under the `spy_on` argument.

```rust
struct EventSpy {
    events: Array<(ContractAddress, Event)>,
}
```
An event spy structure, along with the events collected so far in the test.
`events` are mutable and can be updated with `fetch_events`.

```rust
struct Event {
    keys: Array<felt252>,
    data: Array<felt252>
}
```

Raw event format (as seen via the RPC-API), can be used for asserting the emitted events.

```rust
enum SpyOn {
    All: (),
    One: ContractAddress,
    Multiple: Array<ContractAddress>
}
```

Allows specifying which contracts you want to capture events from.

## Implemented traits

### EventFetcher

```rust
trait EventFetcher {
    fn fetch_events(ref self: EventSpy);
}
```

Allows to update the structs' events field, from the spied contracts.

### EventAssertions

```rust
trait EventAssertions<T, impl TEvent: starknet::Event<T>, impl TDrop: Drop<T>> {
    fn assert_emitted(ref self: EventSpy, events: @Array<(ContractAddress, T)>);
    fn assert_not_emitted(ref self: EventSpy, events: @Array<(ContractAddress, T)>);
}
```

Allows to assert the expected events emission (or lack thereof), in the scope of the spy.