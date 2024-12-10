use indoc::indoc;
use std::path::Path;
use test_utils::runner::{assert_case_output_contains, assert_failed, assert_passed, Contract};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
fn spy_messages_to_l1_simple() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::{ContractAddress, EthAddress};
            use snforge_std::{
                declare, ContractClassTrait, DeclareResultTrait,
                spy_messages_to_l1,
                MessageToL1, MessageToL1SpyAssertionsTrait
            };

            #[starknet::interface]
            trait IMessageToL1Checker<TContractState> {
                fn send_message(ref self: TContractState, some_data: Array<felt252>, to_address: EthAddress);
            }

            fn deploy_message_to_l1_checker()  -> IMessageToL1CheckerDispatcher {
               let declared = declare("MessageToL1Checker").unwrap().contract_class();
               let (contract_address, _) = declared.deploy(@array![]).unwrap();

               IMessageToL1CheckerDispatcher { contract_address }
            }

            #[test]
            fn spy_messages_to_l1_simple() {
               let message_to_l1_checker = deploy_message_to_l1_checker();

               let mut spy = spy_messages_to_l1();
               message_to_l1_checker.send_message(
                    array![123, 321, 420],
                    0x123.try_into().unwrap()
               );

               spy.assert_sent(
                    @array![
                        (
                            message_to_l1_checker.contract_address,
                            MessageToL1 {
                                to_address: 0x123.try_into().unwrap(),
                                payload: array![123, 321, 420]
                            }
                        )
                    ]
               );
            }
        "#
        ),
        Contract::from_code_path(
            "MessageToL1Checker".to_string(),
            Path::new("tests/data/contracts/message_to_l1_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn spy_messages_to_l1_fails() {
    let test = test_case!(indoc!(
        r"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::ContractAddress;
            use snforge_std::{
                declare, ContractClassTrait, 
                spy_messages_to_l1, 
                MessageToL1, MessageToL1SpyAssertionsTrait
            };
            

            #[test]
            fn assert_sent_fails() {
                let mut spy = spy_messages_to_l1();
                spy.assert_sent(
                    @array![
                        (
                            0x123.try_into().unwrap(),
                            MessageToL1 {
                                to_address: 0x123.try_into().unwrap(), 
                                payload: array![0x123, 0x420]
                            }
                        )
                    ]
               );
            }
        "
    ));

    let result = run_test_case(&test);

    assert_failed(&result);
    assert_case_output_contains(
        &result,
        "assert_sent_fails",
        "Message with matching data and",
    );
    assert_case_output_contains(
        &result,
        "assert_sent_fails",
        "receiver was not emitted from",
    );
}

#[test]
fn expect_three_messages_while_two_sent() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::{ContractAddress, EthAddress};
            use snforge_std::{
                declare, ContractClassTrait, DeclareResultTrait,
                spy_messages_to_l1,
                MessageToL1, MessageToL1SpyAssertionsTrait
            };

            #[starknet::interface]
            trait IMessageToL1Checker<TContractState> {
                fn send_message(ref self: TContractState, some_data: Array<felt252>, to_address: EthAddress);
            }

            fn deploy_message_to_l1_checker()  -> IMessageToL1CheckerDispatcher {
               let declared = declare("MessageToL1Checker").unwrap().contract_class();
               let (contract_address, _) = declared.deploy(@array![]).unwrap();

               IMessageToL1CheckerDispatcher { contract_address }
            }

            #[test]
            fn expect_three_messages_while_two_were_sent() {
               let message_to_l1_checker = deploy_message_to_l1_checker();

               let mut spy = spy_messages_to_l1();
               message_to_l1_checker.send_message(
                    array![123, 321, 420],
                    0x123.try_into().unwrap()
               );
               message_to_l1_checker.send_message(
                    array![420, 123, 321],
                    0x321.try_into().unwrap()
               );

               spy.assert_sent(
                    @array![
                        (
                            message_to_l1_checker.contract_address,
                            MessageToL1 {
                                to_address: 0x123.try_into().unwrap(),
                                payload: array![123, 321, 420]
                            }
                        ),
                        (
                            message_to_l1_checker.contract_address,
                            MessageToL1 {
                                to_address: 0x321.try_into().unwrap(),
                                payload: array![420, 123, 321]
                            }
                        ),
                        (
                            message_to_l1_checker.contract_address,
                            MessageToL1 {
                                to_address: 0x456.try_into().unwrap(),
                                payload: array![567, 8910, 111213]
                            }
                        ),
                    ]
               );
            }
        "#
        ),
        Contract::from_code_path(
            "MessageToL1Checker".to_string(),
            Path::new("tests/data/contracts/message_to_l1_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_failed(&result);
    assert_case_output_contains(
        &result,
        "expect_three_messages_while_two_were_sent",
        "Message with matching data and",
    );
    assert_case_output_contains(
        &result,
        "expect_three_messages_while_two_were_sent",
        "receiver was not emitted",
    );
}

#[test]
fn expect_two_messages_while_three_sent() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use starknet::{ContractAddress, EthAddress};
            use snforge_std::{
                ContractClassTrait, DeclareResultTrait, declare, spy_messages_to_l1,
                MessageToL1, MessageToL1SpyAssertionsTrait
            };
            use traits::Into;

            #[starknet::interface]
            trait IMessageToL1Checker<TContractState> {
                fn send_message(ref self: TContractState, some_data: Array<felt252>, to_address: EthAddress);
            }

            fn deploy_message_to_l1_checker()  -> IMessageToL1CheckerDispatcher {
               let declared = declare("MessageToL1Checker").unwrap().contract_class();
               let (contract_address, _) = declared.deploy(@array![]).unwrap();

               IMessageToL1CheckerDispatcher { contract_address }
            }

            #[test]
            fn expect_two_messages_while_three_sent() {
               let message_to_l1_checker = deploy_message_to_l1_checker();

               let mut spy = spy_messages_to_l1();
               message_to_l1_checker.send_message(
                    array![123, 321, 420],
                    0x123.try_into().unwrap()
               );
               message_to_l1_checker.send_message(
                    array![420, 123, 321],
                    0x321.try_into().unwrap()
               );
               message_to_l1_checker.send_message(
                    array![567, 8910, 111213],
                    0x456.try_into().unwrap()
               );

               spy.assert_sent(
                    @array![
                        (
                            message_to_l1_checker.contract_address,
                            MessageToL1 {
                                to_address: 0x123.try_into().unwrap(),
                                payload: array![123, 321, 420]
                            }
                        ),
                        (
                            message_to_l1_checker.contract_address,
                            MessageToL1 {
                                to_address: 0x321.try_into().unwrap(),
                                payload: array![420, 123, 321]
                            }
                        )
                    ]
               );
            }
        "#
        ),
        Contract::from_code_path(
            "MessageToL1Checker".to_string(),
            Path::new("tests/data/contracts/message_to_l1_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn message_sent_but_wrong_data_asserted() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use starknet::{ContractAddress, EthAddress};
            use snforge_std::{
                declare, ContractClassTrait, DeclareResultTrait,
                spy_messages_to_l1,
                MessageToL1, MessageToL1SpyAssertionsTrait
            };

            #[starknet::interface]
            trait IMessageToL1Checker<TContractState> {
                fn send_message(ref self: TContractState, some_data: Array<felt252>, to_address: EthAddress);
            }

            fn deploy_message_to_l1_checker()  -> IMessageToL1CheckerDispatcher {
               let declared = declare("MessageToL1Checker").unwrap().contract_class();
               let (contract_address, _) = declared.deploy(@array![]).unwrap();

               IMessageToL1CheckerDispatcher { contract_address }
            }

            #[test]
            fn message_sent_but_wrong_data_asserted() {
               let message_to_l1_checker = deploy_message_to_l1_checker();

               let mut spy = spy_messages_to_l1();
               message_to_l1_checker.send_message(
                    array![123, 321, 420],
                    0x123.try_into().unwrap()
               );

               spy.assert_sent(
                    @array![
                        (
                            message_to_l1_checker.contract_address,
                            MessageToL1 {
                                to_address: 0x123.try_into().unwrap(),
                                payload: array![420, 321, 123]
                            }
                        )
                    ]
               );
            }
        "#
        ),
        Contract::from_code_path(
            "MessageToL1Checker".to_string(),
            Path::new("tests/data/contracts/message_to_l1_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_failed(&result);
    assert_case_output_contains(
        &result,
        "message_sent_but_wrong_data_asserted",
        "Message with matching data and",
    );
    assert_case_output_contains(
        &result,
        "message_sent_but_wrong_data_asserted",
        "receiver was not emitted from",
    );
}

#[test]
fn assert_not_sent_pass() {
    let test = test_case!(
        indoc!(
            r"
            use array::ArrayTrait;
            use starknet::{ContractAddress, EthAddress};
            use snforge_std::{
                declare, spy_messages_to_l1,
                MessageToL1, MessageToL1SpyAssertionsTrait
            };

            #[test]
            fn assert_not_sent_pass() {
               let mut spy = spy_messages_to_l1();
               spy.assert_not_sent(
                    @array![
                        (
                            0x123.try_into().unwrap(),
                            MessageToL1 {
                                to_address: 0x123.try_into().unwrap(),
                                payload: ArrayTrait::new()
                            }
                        )
                     ]
                );
            }
        "
        ),
        Contract::from_code_path(
            "MessageToL1Checker".to_string(),
            Path::new("tests/data/contracts/message_to_l1_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn assert_not_sent_fails() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use starknet::{ContractAddress, EthAddress};
            use snforge_std::{
                ContractClassTrait, DeclareResultTrait, declare, spy_messages_to_l1,
                MessageToL1, MessageToL1SpyAssertionsTrait
            };

            #[starknet::interface]
            trait IMessageToL1Checker<TContractState> {
                fn send_message(ref self: TContractState, some_data: Array<felt252>, to_address: EthAddress);
            }

            fn deploy_message_to_l1_checker()  -> IMessageToL1CheckerDispatcher {
               let declared = declare("MessageToL1Checker").unwrap().contract_class();
               let (contract_address, _) = declared.deploy(@array![]).unwrap();

               IMessageToL1CheckerDispatcher { contract_address }
            }

            #[test]
            fn assert_not_sent_fails() {
               let message_to_l1_checker = deploy_message_to_l1_checker();

               let mut spy = spy_messages_to_l1();
               message_to_l1_checker.send_message(
                    array![123, 321, 420],
                    0x123.try_into().unwrap()
               );

               spy.assert_not_sent(
                    @array![
                        (
                            message_to_l1_checker.contract_address,
                            MessageToL1 {
                                to_address: 0x123.try_into().unwrap(),
                                payload: array![123, 321, 420]
                            }
                        )
                    ]
               );
            }
        "#
        ),
        Contract::from_code_path(
            "MessageToL1Checker".to_string(),
            Path::new("tests/data/contracts/message_to_l1_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_failed(&result);
    assert_case_output_contains(
        &result,
        "assert_not_sent_fails",
        "Message with matching data and",
    );
    assert_case_output_contains(&result, "assert_not_sent_fails", "receiver was sent from");
}

#[test]
fn test_filtering() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::{ContractAddress, EthAddress};
            use snforge_std::{
                declare, ContractClassTrait, DeclareResultTrait, spy_messages_to_l1,
                MessageToL1, MessageToL1SpyAssertionsTrait, MessageToL1FilterTrait, MessageToL1SpyTrait
            };


            fn deploy_message_to_l1_checkers()  -> (IMessageToL1CheckerDispatcher, IMessageToL1CheckerDispatcher) {
               let declared = declare("MessageToL1Checker").unwrap().contract_class();
               let (contract_address_1, _) = declared.deploy(@array![]).unwrap();
               let (contract_address_2, _) = declared.deploy(@array![]).unwrap();

               (
                    IMessageToL1CheckerDispatcher { contract_address: contract_address_1 },
                    IMessageToL1CheckerDispatcher { contract_address: contract_address_2 }
               )
            }

            #[starknet::interface]
            trait IMessageToL1Checker<TContractState> {
                fn send_message(ref self: TContractState, some_data: Array<felt252>, to_address: EthAddress);
            }

            #[test]
            fn filter_events() {
                let (first_dispatcher, second_dispatcher) = deploy_message_to_l1_checkers();
                let first_address = first_dispatcher.contract_address;
                let second_address = second_dispatcher.contract_address;

                let mut spy = spy_messages_to_l1();

                first_dispatcher.send_message(
                    array![123, 421, 420],
                    0x123.try_into().unwrap()
                 );
                second_dispatcher.send_message(
                    array![123, 124, 420],
                    0x125.try_into().unwrap()
                );

                let messages_from_first_address = spy.get_messages().sent_by(first_address);
                let messages_from_second_address = spy.get_messages().sent_by(second_address);

                let (from, message) = messages_from_first_address.messages.at(0);
                assert!(from == @first_address, "Sent from wrong address");
                assert!(message.payload.len() == 3, "There should be 3 items in the data");
                assert!(*message.payload.at(1) == 421, "Expected 421 in payload");

                let (from, message) = messages_from_second_address.messages.at(0);
                assert!(from == @second_address, "Sent from wrong address");
                assert!(message.payload.len() == 3, "There should be 3 items in the data");
                assert!(*message.payload.at(1) == 124, "Expected 124 in payload");
            }
        "#,
        ),
        Contract::from_code_path(
            "MessageToL1Checker".to_string(),
            Path::new("tests/data/contracts/message_to_l1_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}
