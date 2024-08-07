# `spy_messages_to_l1`

> `fn spy_messages_to_l1() -> MessageToL1Spy`

Creates `MessageToL1Spy` instance that spies on all messages sent to L1 after its creation.

```rust
struct MessageToL1Spy {
    // ..
}
```
Message spy structure allowing to get messages emitted only after its creation.

```rust
struct MessagesToL1 {
    messages: Array<(ContractAddress, MessageToL1)>
}
```
A wrapper structure on an array of messages to handle filtering smoothly.
`messages` is an array of `(l2_sender_address, message)` tuples. 

```rust
struct MessageToL1 {
    /// An ethereum address where the message is destined to go
    to_address: EthAddress,
    /// Actual payload which will be delivered to L1 contract
    payload: Array<felt252>
}

```
Raw message to L1 format (as seen via the RPC-API), can be used for asserting the sent messages.

## Implemented traits

### MessageToL1SpyTrait

```rust
trait MessageToL1SpyTrait {
    /// Gets all messages given [`MessageToL1Spy`] spies for.
    fn get_messages(ref self: MessageToL1Spy) -> MessagesToL1;
}
```
Gets all messages since the creation of the given `MessageToL1Spy`. 

### MessageToL1SpyAssertionsTrait

```rust
trait MessageToL1SpyAssertionsTrait {
    fn assert_sent(ref self: MessageToL1Spy, messages: @Array<(ContractAddress, MessageToL1)>);
    fn assert_not_sent(ref self: MessageToL1Spy, messages: @Array<(ContractAddress, MessageToL1)>);
}
```
Allows to assert the expected sent messages (or lack thereof), in the scope of `MessageToL1Spy` structure.

### MessageToL1FilterTrait

```rust
trait MessageToL1FilterTrait {
    /// Filter messages emitted by a sender of a given [`ContractAddress`]
    fn sent_by(self: @MessagesToL1, contract_address: ContractAddress) -> MessagesToL1;
    /// Filter messages emitted by a receiver of a given ethereum address
    fn sent_to(self: @MessagesToL1, to_address: EthAddress) -> MessagesToL1;
}
```

Filters messages emitted by a given `ContractAddress`, or sent to given `EthAddress`.
