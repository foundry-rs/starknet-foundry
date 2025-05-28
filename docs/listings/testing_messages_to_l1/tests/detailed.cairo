use snforge_std::{
    ContractClassTrait, DeclareResultTrait, MessageToL1, MessageToL1FilterTrait,
    MessageToL1SpyAssertionsTrait, MessageToL1SpyTrait, declare, spy_messages_to_l1,
};
use starknet::EthAddress;
use testing_messages_to_l1::{IMessageSenderDispatcher, IMessageSenderDispatcherTrait};

#[test]
fn test_spying_l1_messages_details() {
    let mut spy = spy_messages_to_l1();

    let contract = declare("MessageSender").unwrap().contract_class();
    let (contract_address, _) = contract.deploy(@array![]).unwrap();

    let dispatcher = IMessageSenderDispatcher { contract_address };

    let receiver_address: felt252 = 0x2137;
    let receiver_l1_address: EthAddress = receiver_address.try_into().unwrap();

    dispatcher.greet_ethereum(receiver_address);

    let messages = spy.get_messages();

    // Use filtering optionally on MessagesToL1 instance
    let messages_from_specific_address = messages.sent_by(contract_address);
    let messages_to_specific_address = messages_from_specific_address.sent_to(receiver_l1_address);

    // Get the messages from the MessagesToL1 structure
    let (from, message) = messages_to_specific_address.messages.at(0);

    // Assert the sender
    assert!(*from == contract_address, "Sent from wrong address");

    // Assert the MessageToL1 fields
    assert!(*message.to_address == receiver_l1_address, "Wrong L1 address of the receiver");
    assert!(message.payload.len() == 1, "There should be 3 items in the data");
    assert!(*message.payload.at(0) == 'hello', "Expected \"hello\" in payload");
}
