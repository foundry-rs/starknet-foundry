use core::array::ArrayTrait;
use core::option::OptionTrait;
use starknet::testing::cheatcode;
use starknet::{ContractAddress, EthAddress};
use super::super::_cheatcode::handle_cheatcode;

/// Creates `MessageToL1Spy` instance that spies on all messages sent to L1
pub fn spy_messages_to_l1() -> MessageToL1Spy {
    let mut message_offset = handle_cheatcode(cheatcode::<'spy_messages_to_l1'>(array![].span()));
    let parsed_message_offset: usize = Serde::<usize>::deserialize(ref message_offset).unwrap();

    MessageToL1Spy { message_offset: parsed_message_offset }
}

/// Raw message to L1 format (as seen via the RPC-API), can be used for asserting the sent messages.
#[derive(Drop, Clone, Serde)]
pub struct MessageToL1 {
    /// An ethereum address where the message is destined to go
    pub to_address: EthAddress,
    /// Actual payload which will be delivered to L1 contract
    pub payload: Array<felt252>
}

/// A message spy structure allowing to get messages emitted only after its creation.
#[derive(Drop, Serde)]
pub struct MessageToL1Spy {
    message_offset: usize
}

/// A wrapper structure on an array of messages to handle filtering smoothly.
#[derive(Drop, Serde)]
pub struct MessagesToL1 {
    pub messages: Array<(ContractAddress, MessageToL1)>
}

pub trait MessageToL1SpyTrait {
    /// Gets all messages given [`MessageToL1Spy`] spies for.
    fn get_messages(ref self: MessageToL1Spy) -> MessagesToL1;
}

impl MessageToL1SpyTraitImpl of MessageToL1SpyTrait {
    fn get_messages(ref self: MessageToL1Spy) -> MessagesToL1 {
        let mut output = handle_cheatcode(
            cheatcode::<'get_messages_to_l1'>(array![self.message_offset.into()].span())
        );
        let messages = Serde::<Array<(ContractAddress, MessageToL1)>>::deserialize(ref output)
            .unwrap();

        MessagesToL1 { messages }
    }
}

pub trait MessageToL1FilterTrait {
    /// Filter messages emitted by a sender of a given [`ContractAddress`]
    fn sent_by(self: @MessagesToL1, contract_address: ContractAddress) -> MessagesToL1;
    /// Filter messages emitted by a receiver of a given ethereum address
    fn sent_to(self: @MessagesToL1, to_address: EthAddress) -> MessagesToL1;
}

impl MessageToL1FilterTraitImpl of MessageToL1FilterTrait {
    fn sent_by(self: @MessagesToL1, contract_address: ContractAddress) -> MessagesToL1 {
        let mut counter = 0;
        let mut new_messages = array![];
        while counter < self.messages.len() {
            let (sent_by, msg) = self.messages.at(counter);
            if *sent_by == contract_address {
                new_messages.append((*sent_by, msg.clone()));
            };
            counter += 1;
        };
        MessagesToL1 { messages: new_messages }
    }
    fn sent_to(self: @MessagesToL1, to_address: EthAddress) -> MessagesToL1 {
        let mut counter = 0;
        let mut new_messages = array![];
        while counter < self.messages.len() {
            let (sent_by, msg) = self.messages.at(counter);
            if *msg.to_address == to_address {
                new_messages.append((*sent_by, msg.clone()));
            };
            counter += 1;
        };
        MessagesToL1 { messages: new_messages }
    }
}

/// Allows to assert the expected sent messages (or lack thereof),
/// in the scope of [`MessageToL1Spy`] structure.
pub trait MessageToL1SpyAssertionsTrait {
    fn assert_sent(ref self: MessageToL1Spy, messages: @Array<(ContractAddress, MessageToL1)>);
    fn assert_not_sent(ref self: MessageToL1Spy, messages: @Array<(ContractAddress, MessageToL1)>);
}

impl MessageToL1SpyAssertionsTraitImpl of MessageToL1SpyAssertionsTrait {
    fn assert_sent(ref self: MessageToL1Spy, messages: @Array<(ContractAddress, MessageToL1)>) {
        let mut i = 0;
        let sent_messages = self.get_messages();

        while i < messages.len() {
            let (from, message) = messages.at(i);
            let sent = is_sent(@sent_messages, from, message);

            if !sent {
                let from: felt252 = (*from).into();
                panic!("Message with matching data and receiver was not emitted from {}", from);
            }
            i += 1;
        };
    }
    fn assert_not_sent(ref self: MessageToL1Spy, messages: @Array<(ContractAddress, MessageToL1)>) {
        let mut i = 0;
        let sent_messages = self.get_messages();

        while i < messages.len() {
            let (from, message) = messages.at(i);
            let emitted = is_sent(@sent_messages, from, message);

            if emitted {
                let from: felt252 = (*from).into();
                panic!("Message with matching data and receiver was sent from {}", from);
            }

            i += 1;
        };
    }
}

fn is_sent(
    messages: @MessagesToL1, expected_sent_by: @ContractAddress, expected_message: @MessageToL1,
) -> bool {
    let mut i = 0;
    let mut is_emitted = false;
    while i < messages.messages.len() {
        let (from, message) = messages.messages.at(i);

        if from == expected_sent_by
            && message.payload == expected_message.payload
            && message.to_address == expected_message.to_address {
            is_emitted = true;
            break;
        };

        i += 1;
    };
    return is_emitted;
}

