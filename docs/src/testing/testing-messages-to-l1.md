# Testing messages to L1

There exists a functionality allowing you to spy on messages sent to L1, similar to [spying events](./testing-events.md).

[Check the appendix for an exact API, structures and traits reference](../appendix/cheatcodes/spy_messages_to_l1.md)

Asserting messages to L1 is much simpler, since they are not wrapped with any structures in Cairo code (they are a plain `felt252` array and an L1 address). 
In `snforge` they are expressed with a structure:

```rust
/// Raw message to L1 format (as seen via the RPC-API), can be used for asserting the sent messages.
struct MessageToL1 {
    /// An ethereum address where the message is destined to go
    to_address: EthAddress,
    /// Actual payload which will be delivered to L1 contract
    payload: Array<felt252>
}
```

Similarly, you can use `snforge` library and call `spy_messages_to_l1()` to initiate a spy:

```rust
use snforge_std::{spy_messages_to_l1};

// ...

#[test]
fn test_spying_l1_messages() {
    let mut spy = spy_messages_to_l1();
    // ...
}
```

With the spy ready to use, you can execute some code, and make the assertions:

1. Either with the spy directly by using `assert_sent`/`assert_not_sent` methods from `MessageToL1SpyAssertionsTrait` trait:

```rust
use snforge_std::{spy_messages_to_l1, MessageToL1SpyAssertionsTrait, MessageToL1};

// ...

#[test]
fn test_spying_l1_messages() {
    let mut spy = spy_messages_to_l1();
    // ...
    spy.assert_sent(
        @array![
            (
                contract_address, // Message sender
                MessageToL1 {     // Message content (receiver and payload)
                    to_address: 0x123.try_into().unwrap(), 
                    payload: array![123, 321, 420]
                }
            )
        ]
    );
}
```

2. Or use the messages' contents directly via `get_messages()` method of the `MessageToL1Spy` trait:

```rust
use snforge_std::{
    spy_messages_to_l1, MessageToL1, 
    MessageToL1SpyAssertionsTrait, 
    MessageToL1FilterTrait, 
    MessageToL1SpyTrait
};

// ...

#[test]
fn test_spying_l1_messages() {
    let mut spy = spy_messages_to_l1();
    
    let messages = spy.get_messages();
    
    // Use filtering optionally on MessagesToL1 instance
    let messages_from_specific_address = messages.sent_by(sender_address);
    let messages_to_specific_address   = messages_from_specific_address.sent_to(receiver_eth_address);

    // Get the messages from the MessagesToL1 structure 
    let (from, message) = messages_to_specific_address.messages.at(0);

    // Assert the sender
    assert!(from == sender_address, "Sent from wrong address");
    // Assert the MessageToL1 fields
    assert!(message.to_address == receiver_eth_address, "Wrong eth address of the receiver");
    assert!(message.payload.len() == 3, "There should be 3 items in the data");
    assert!(*message.payload.at(1) == 421, "Expected 421 in payload");
}
```
