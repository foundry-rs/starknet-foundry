use starknet::EthAddress;
use snforge_std::{
    declare, ContractClassTrait, DeclareResultTrait, spy_messages_to_l1,
    MessageToL1SpyAssertionsTrait, MessageToL1,
};

use testing_messages_to_l1::{IMessageSenderDispatcher, IMessageSenderDispatcherTrait};

#[test]
fn test_spying_l1_messages() {
    let mut spy = spy_messages_to_l1();

    let contract = declare("MessageSender").unwrap().contract_class();
    let (contract_address, _) = contract.deploy(@array![]).unwrap();

    let dispatcher = IMessageSenderDispatcher { contract_address };

    let receiver_address: felt252 = 0x2137;
    dispatcher.greet_ethereum(receiver_address);

    let expected_payload = array!['hello'];
    let receiver_l1_address: EthAddress = receiver_address.try_into().unwrap();

    spy
        .assert_sent(
            @array![
                (
                    contract_address, // Message sender
                    MessageToL1 { // Message content (receiver and payload)
                        to_address: receiver_l1_address, payload: expected_payload
                    }
                )
            ]
        );
}
