# `spy_events`

> `fn spy_events() -> EventSpy`

Creates `EventSpy` instance which spies on events emitted after its creation.

```rust
struct EventSpy {
    ...
}
```
An event spy structure.

```rust
struct Events {
    events: Array<(ContractAddress, Event)>
}
```
A wrapper structure on an array of events to handle event filtering. 

```rust
struct Event {
    keys: Array<felt252>,
    data: Array<felt252>
}
```
Raw event format (as seen via the RPC-API), can be used for asserting the emitted events.

## Implemented traits

### EventSpyTrait

```rust
trait EventSpyTrait {
    fn get_events(ref self: EventSpy) -> Events;
}
```
Gets all events since the creation of the given `EventSpy`. 

### EventSpyAssertionsTrait

```rust
trait EventSpyAssertionsTrait<T, impl TEvent: starknet::Event<T>, impl TDrop: Drop<T>> {
    fn assert_emitted(ref self: EventSpy, events: @Array<(ContractAddress, T)>);
    fn assert_not_emitted(ref self: EventSpy, events: @Array<(ContractAddress, T)>);
}
```
Allows to assert the expected events emission (or lack thereof), in the scope of the `EventSpy` structure.

### EventsFilterTrait

```rust
trait EventsFilterTrait {
    fn emitted_by(self: @Events, contract_address: ContractAddress) -> Events;
}
```
Filters events emitted by a given `ContractAddress`.
