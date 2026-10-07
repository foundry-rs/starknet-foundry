use crate::utils::runner::{Contract, assert_case_output_contains, assert_failed, assert_passed};
use crate::utils::running_tests::run_test_case;
use crate::utils::test_case;
use forge_runner::forge_config::ForgeTrackedResource;
use indoc::{formatdoc, indoc};
use shared::test_utils::node_url::node_rpc_url;
use std::path::Path;

#[test]
fn spy_events_simple() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::ContractAddress;
            use snforge_std::{
                declare, ContractClassTrait, DeclareResultTrait, spy_events, Event,
                EventSpy, EventSpyTrait, EventSpyAssertionsTrait, EventsFilterTrait
            };

            #[starknet::interface]
            trait ISpyEventsChecker<TContractState> {
                fn emit_one_event(ref self: TContractState, some_data: felt252);
            }

            #[starknet::contract]
            mod SpyEventsChecker {
                use starknet::ContractAddress;

                #[storage]
                struct Storage {}

                #[event]
                #[derive(Drop, starknet::Event)]
                enum Event {
                    FirstEvent: FirstEvent
                }

                #[derive(Drop, starknet::Event)]
                struct FirstEvent {
                    some_data: felt252
                }
            }

            #[test]
            fn spy_events_simple() {
                let contract = declare("SpyEventsChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

                let mut spy = spy_events();
                // assert(spy._event_offset == 0, 'Event offset should be 0'); TODO(#2765)
                dispatcher.emit_one_event(123);

                spy.assert_emitted(@array![
                    (
                        contract_address,
                        SpyEventsChecker::Event::FirstEvent(
                            SpyEventsChecker::FirstEvent { some_data: 123 }
                        )
                    )
                ]);
                assert(spy.get_events().events.len() == 1, 'There should be one event');
            }
        "#
        ),
        Contract::from_code_path(
            "contract::SpyEventsChecker".to_string(),
            Path::new("tests/data/contracts/spy_events_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn spy_events_emitted_by_library_call() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use snforge_std::{
                declare, ContractClassTrait, DeclareResultTrait, spy_events, test_address, Event,
                EventSpy, EventSpyAssertionsTrait
            };

            #[starknet::interface]
            trait ISpyEventsChecker<TContractState> {
                fn emit_one_event(ref self: TContractState, some_data: felt252);
            }

            #[starknet::contract]
            mod SpyEventsChecker {
                #[storage]
                struct Storage {}

                #[event]
                #[derive(Drop, starknet::Event)]
                enum Event {
                    FirstEvent: FirstEvent
                }

                #[derive(Drop, starknet::Event)]
                struct FirstEvent {
                    some_data: felt252
                }
            }

            #[test]
            fn spy_events_emitted_by_library_call() {
                let contract = declare("SpyEventsChecker").unwrap().contract_class();
                let dispatcher = ISpyEventsCheckerLibraryDispatcher {
                    class_hash: contract.class_hash.clone()
                };

                let mut spy = spy_events();
                dispatcher.emit_one_event(123);

                spy.assert_emitted(@array![
                    (
                        test_address(),
                        SpyEventsChecker::Event::FirstEvent(
                            SpyEventsChecker::FirstEvent { some_data: 123 }
                        )
                    )
                ]);
            }
        "#
        ),
        Contract::from_code_path(
            "contract::SpyEventsChecker".to_string(),
            Path::new("tests/data/contracts/spy_events_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn spy_events_emitted_by_library_call_fails() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use snforge_std::{
                declare, ContractClassTrait, DeclareResultTrait, spy_events, test_address, Event,
                EventSpy, EventSpyAssertionsTrait
            };

            #[starknet::interface]
            trait ISpyEventsChecker<TContractState> {
                fn emit_one_event(ref self: TContractState, some_data: felt252);
            }

            #[starknet::contract]
            mod SpyEventsChecker {
                #[storage]
                struct Storage {}

                #[event]
                #[derive(Drop, starknet::Event)]
                enum Event {
                    FirstEvent: FirstEvent
                }

                #[derive(Drop, starknet::Event)]
                struct FirstEvent {
                    some_data: felt252
                }
            }

            #[test]
            fn spy_events_emitted_by_library_call_fails() {
                let contract = declare("SpyEventsChecker").unwrap().contract_class();
                let dispatcher = ISpyEventsCheckerLibraryDispatcher {
                    class_hash: contract.class_hash.clone()
                };

                let mut spy = spy_events();
                dispatcher.emit_one_event(123);

                spy.assert_emitted(@array![
                    (
                        test_address(),
                        SpyEventsChecker::Event::FirstEvent(
                            SpyEventsChecker::FirstEvent { some_data: 456 }
                        )
                    )
                ]);
            }
        "#
        ),
        Contract::from_code_path(
            "contract::SpyEventsChecker".to_string(),
            Path::new("tests/data/contracts/spy_events_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_failed(&result);
    assert_case_output_contains(
        &result,
        "spy_events_emitted_by_library_call_fails",
        "Event { keys: [0xa7f959d3eee3641c46990c878d8b5c3072bd06e41c69736b51b45017c5dad6], data: [0x1c8] } was not emitted from contract at",
    );
}

#[test]
fn assert_emitted_fails() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::ContractAddress;
            use snforge_std::{
                declare, DeclareResultTrait, ContractClassTrait, spy_events, Event,
                EventSpy, EventSpyTrait, EventSpyAssertionsTrait, EventsFilterTrait
            };

            #[starknet::interface]
            trait ISpyEventsChecker<TContractState> {
                fn do_not_emit(ref self: TContractState);
            }

            #[starknet::contract]
            mod SpyEventsChecker {
                use starknet::ContractAddress;

                #[storage]
                struct Storage {}

                #[event]
                #[derive(Drop, starknet::Event)]
                enum Event {
                    FirstEvent: FirstEvent
                }

                #[derive(Drop, starknet::Event)]
                struct FirstEvent {
                    some_data: felt252
                }
            }

            #[test]
            fn assert_emitted_fails() {
                let contract = declare("SpyEventsChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

                let mut spy = spy_events();
                dispatcher.do_not_emit();

                spy.assert_emitted(@array![
                    (
                        contract_address,
                        SpyEventsChecker::Event::FirstEvent(
                            SpyEventsChecker::FirstEvent { some_data: 123 }
                        )
                    )
                ]);
            }
        "#
        ),
        Contract::from_code_path(
            "contract::SpyEventsChecker".to_string(),
            Path::new("tests/data/contracts/spy_events_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_failed(&result);
    assert_case_output_contains(
        &result,
        "assert_emitted_fails",
        "Event FirstEvent { some_data: 0x7b } was not emitted from contract::SpyEventsChecker at",
    );
}

#[test]
fn expect_three_events_while_two_emitted() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use traits::Into;
            use starknet::contract_address_const;
            use starknet::ContractAddress;
            use snforge_std::{
                declare, ContractClassTrait, DeclareResultTrait, spy_events, Event,
                EventSpy, EventSpyTrait, EventSpyAssertionsTrait, EventsFilterTrait
            };

            #[starknet::interface]
            trait ISpyEventsChecker<TContractState> {
                fn emit_two_events(ref self: TContractState, some_data: felt252, some_more_data: ContractAddress);
            }

            #[starknet::contract]
            mod SpyEventsChecker {
                use starknet::ContractAddress;

                #[storage]
                struct Storage {}

                #[event]
                #[derive(Drop, starknet::Event)]
                enum Event {
                    FirstEvent: FirstEvent,
                    SecondEvent: SecondEvent,
                    ThirdEvent: ThirdEvent,
                }

                #[derive(Drop, starknet::Event)]
                struct FirstEvent {
                    some_data: felt252
                }

                #[derive(Drop, starknet::Event)]
                struct SecondEvent {
                    some_data: felt252,
                    #[key]
                    some_more_data: ContractAddress
                }

                #[derive(Drop, starknet::Event)]
                struct ThirdEvent {
                    some_data: felt252,
                    some_more_data: ContractAddress,
                    even_more_data: u256
                }
            }

            #[test]
            fn expect_three_events_while_two_emitted() {
                let contract = declare("SpyEventsChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@array![]).unwrap();
                let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

                let some_data = 456;
                let some_more_data = contract_address_const::<789>();
                let even_more_data = 0;

                let mut spy = spy_events();
                dispatcher.emit_two_events(some_data, some_more_data);

                spy.assert_emitted(@array![
                    (
                        contract_address,
                        SpyEventsChecker::Event::FirstEvent(
                            SpyEventsChecker::FirstEvent { some_data }
                        )
                    ),
                    (
                        contract_address,
                        SpyEventsChecker::Event::SecondEvent(
                            SpyEventsChecker::SecondEvent { some_data, some_more_data }
                        )
                    ),
                    (
                        contract_address,
                        SpyEventsChecker::Event::ThirdEvent(
                            SpyEventsChecker::ThirdEvent { some_data, some_more_data, even_more_data }
                        )
                    )
                ]);
            }
        "#
        ),
        Contract::from_code_path(
            "contract::SpyEventsChecker".to_string(),
            Path::new("tests/data/contracts/spy_events_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_failed(&result);
    assert_case_output_contains(
        &result,
        "expect_three_events_while_two_emitted",
        "contract::SpyEventsChecker",
    );
    assert_case_output_contains(
        &result,
        "expect_three_events_while_two_emitted",
        "Event ThirdEvent {",
    );
}

#[test]
fn expect_two_events_while_three_emitted() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use traits::Into;
            use starknet::contract_address_const;
            use starknet::ContractAddress;
            use snforge_std::{
                declare, ContractClassTrait, DeclareResultTrait, spy_events, Event,
                EventSpy, EventSpyTrait, EventSpyAssertionsTrait, EventsFilterTrait
            };

            #[starknet::interface]
            trait ISpyEventsChecker<TContractState> {
                fn emit_three_events(
                    ref self: TContractState,
                    some_data: felt252,
                    some_more_data: ContractAddress,
                    even_more_data: u256
                );
            }

            #[starknet::contract]
            mod SpyEventsChecker {
                use starknet::ContractAddress;

                #[storage]
                struct Storage {}

                #[event]
                #[derive(Drop, starknet::Event)]
                enum Event {
                    FirstEvent: FirstEvent,
                    ThirdEvent: ThirdEvent,
                }

                #[derive(Drop, starknet::Event)]
                struct FirstEvent {
                    some_data: felt252
                }

                #[derive(Drop, starknet::Event)]
                struct ThirdEvent {
                    some_data: felt252,
                    some_more_data: ContractAddress,
                    even_more_data: u256
                }
            }

            #[test]
            fn expect_two_events_while_three_emitted() {
                let contract = declare("SpyEventsChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@array![]).unwrap();
                let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

                let some_data = 456;
                let some_more_data = contract_address_const::<789>();
                let even_more_data = u256 { low: 1, high: 0 };

                let mut spy = spy_events();
                dispatcher.emit_three_events(some_data, some_more_data, even_more_data);

                spy.assert_emitted(@array![
                    (
                        contract_address,
                        SpyEventsChecker::Event::FirstEvent(
                            SpyEventsChecker::FirstEvent { some_data }
                        ),
                    ),
                    (
                        contract_address,
                        SpyEventsChecker::Event::ThirdEvent(
                            SpyEventsChecker::ThirdEvent { some_data, some_more_data, even_more_data }
                        )
                    )
                ]);
            }
        "#
        ),
        Contract::from_code_path(
            "contract::SpyEventsChecker".to_string(),
            Path::new("tests/data/contracts/spy_events_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn event_emitted_wrong_data_asserted() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::ContractAddress;
            use snforge_std::{
                declare, ContractClassTrait, DeclareResultTrait, spy_events, Event,
                EventSpy, EventSpyTrait, EventSpyAssertionsTrait, EventsFilterTrait
            };

            #[starknet::interface]
            trait ISpyEventsChecker<TContractState> {
                fn emit_one_event(ref self: TContractState, some_data: felt252);
            }

            #[starknet::contract]
            mod SpyEventsChecker {
                use starknet::ContractAddress;

                #[storage]
                struct Storage {}

                #[event]
                #[derive(Drop, starknet::Event)]
                enum Event {
                    FirstEvent: FirstEvent
                }

                #[derive(Drop, starknet::Event)]
                struct FirstEvent {
                    some_data: felt252
                }
            }

            #[test]
            fn event_emitted_wrong_data_asserted() {
                let contract = declare("SpyEventsChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

                let mut spy = spy_events();
                dispatcher.emit_one_event(123);

                spy.assert_emitted(@array![
                    (
                        contract_address,
                        SpyEventsChecker::Event::FirstEvent(
                            SpyEventsChecker::FirstEvent { some_data: 124 }
                        ),
                    )
                ]);
            }
        "#
        ),
        Contract::from_code_path(
            "contract::SpyEventsChecker".to_string(),
            Path::new("tests/data/contracts/spy_events_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_failed(&result);
    assert_case_output_contains(
        &result,
        "event_emitted_wrong_data_asserted",
        "Event FirstEvent { some_data: 0x7c } was not emitted from contract::SpyEventsChecker at",
    );
}

#[test]
fn emit_unnamed_event() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use traits::Into;
            use starknet::contract_address_const;
            use starknet::ContractAddress;
            use snforge_std::{
                declare, ContractClassTrait, DeclareResultTrait, spy_events, Event,
                EventSpy, EventSpyTrait, EventSpyAssertionsTrait, EventsFilterTrait
            };

            #[starknet::interface]
            trait ISpyEventsChecker<TContractState> {
                fn emit_event_syscall(ref self: TContractState, some_key: felt252, some_data: felt252);
            }

            #[test]
            fn emit_unnamed_event() {
                let contract = declare("SpyEventsChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@array![]).unwrap();
                let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

                let mut spy = spy_events();
                dispatcher.emit_event_syscall(123, 456);

                spy.assert_emitted(@array![
                    (
                        contract_address,
                        Event { keys: array![123], data: array![456] }
                    )
                ]);
            }
        "#
        ),
        Contract::from_code_path(
            "contract::SpyEventsChecker".to_string(),
            Path::new("tests/data/contracts/spy_events_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn emit_unnamed_event_fails() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::ContractAddress;
            use snforge_std::{
                declare, ContractClassTrait, DeclareResultTrait, spy_events, Event,
                EventSpyAssertionsTrait
            };

            #[starknet::interface]
            trait ISpyEventsChecker<TContractState> {
                fn emit_event_syscall(ref self: TContractState, some_key: felt252, some_data: felt252);
            }

            #[test]
            fn emit_unnamed_event_fails() {
                let contract = declare("SpyEventsChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@array![]).unwrap();
                let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

                let mut spy = spy_events();
                dispatcher.emit_event_syscall(123, 456);

                spy.assert_emitted(@array![
                    (contract_address, Event { keys: array![123], data: array![457] })
                ]);
            }
        "#
        ),
        Contract::from_code_path(
            "contract::SpyEventsChecker".to_string(),
            Path::new("tests/data/contracts/spy_events_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_failed(&result);
    assert_case_output_contains(
        &result,
        "emit_unnamed_event_fails",
        "Event { keys: [0x7b], data: [0x1c9] } was not emitted from contract::SpyEventsChecker at",
    );
}

#[test]
fn assert_not_emitted_pass() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::ContractAddress;
            use snforge_std::{
                declare, ContractClassTrait, DeclareResultTrait, spy_events, Event,
                EventSpy, EventSpyTrait, EventSpyAssertionsTrait, EventsFilterTrait
            };

            #[starknet::interface]
            trait ISpyEventsChecker<TContractState> {
                fn do_not_emit(ref self: TContractState);
            }

            #[starknet::contract]
            mod SpyEventsChecker {
                use starknet::ContractAddress;

                #[storage]
                struct Storage {}

                #[event]
                #[derive(Drop, starknet::Event)]
                enum Event {
                    FirstEvent: FirstEvent,
                }

                #[derive(Drop, starknet::Event)]
                struct FirstEvent {
                    some_data: felt252
                }
            }

            #[test]
            fn assert_not_emitted_pass() {
                let contract = declare("SpyEventsChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

                let mut spy = spy_events();
                dispatcher.do_not_emit();

                spy.assert_not_emitted(@array![
                    (
                        contract_address,
                        SpyEventsChecker::Event::FirstEvent(
                            SpyEventsChecker::FirstEvent { some_data: 123 }
                        )
                    )
                ]);
            }
        "#
        ),
        Contract::from_code_path(
            "contract::SpyEventsChecker".to_string(),
            Path::new("tests/data/contracts/spy_events_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn assert_not_emitted_fails() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::ContractAddress;
            use snforge_std::{
                declare, DeclareResultTrait, ContractClassTrait, spy_events, Event,
                EventSpy, EventSpyTrait, EventSpyAssertionsTrait, EventsFilterTrait
            };

            #[starknet::interface]
            trait ISpyEventsChecker<TContractState> {
                fn emit_one_event(ref self: TContractState, some_data: felt252);
            }

            #[starknet::contract]
            mod SpyEventsChecker {
                use starknet::ContractAddress;

                #[storage]
                struct Storage {}

                #[event]
                #[derive(Drop, starknet::Event)]
                enum Event {
                    FirstEvent: FirstEvent
                }

                #[derive(Drop, starknet::Event)]
                struct FirstEvent {
                    some_data: felt252
                }
            }

            #[test]
            fn assert_not_emitted_fails() {
                let contract = declare("SpyEventsChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

                let mut spy = spy_events();
                dispatcher.emit_one_event(123);

                spy.assert_not_emitted(@array![
                    (
                        contract_address,
                        SpyEventsChecker::Event::FirstEvent(
                            SpyEventsChecker::FirstEvent { some_data: 123 }
                        )
                    )
                ]);
            }
        "#
        ),
        Contract::from_code_path(
            "contract::SpyEventsChecker".to_string(),
            Path::new("tests/data/contracts/spy_events_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_failed(&result);
    assert_case_output_contains(
        &result,
        "assert_not_emitted_fails",
        "Event FirstEvent { some_data: 0x7b } was emitted from contract::SpyEventsChecker at",
    );
}

#[test]
fn capture_cairo0_event() {
    let test = test_case!(
        formatdoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::{{ContractAddress, contract_address_const}};
            use snforge_std::{{
                declare, ContractClassTrait, DeclareResultTrait, spy_events, Event,
                EventSpy, EventSpyTrait, EventSpyAssertionsTrait, EventsFilterTrait
            }};

            #[starknet::interface]
            trait ISpyEventsChecker<TContractState> {{
                fn emit_one_event(ref self: TContractState, some_data: felt252);
                fn test_cairo0_event_collection(ref self: TContractState, cairo0_address: felt252);
            }}

            #[starknet::contract]
            mod SpyEventsChecker {{
                use starknet::ContractAddress;

                #[storage]
                struct Storage {{}}

                #[event]
                #[derive(Drop, starknet::Event)]
                enum Event {{
                    FirstEvent: FirstEvent,
                    my_event: Cairo0Event,
                }}

                #[derive(Drop, starknet::Event)]
                struct FirstEvent {{
                    some_data: felt252
                }}

                #[derive(Drop, starknet::Event)]
                struct Cairo0Event {{
                    some_data: felt252
                }}
            }}

            #[test]
            #[fork(url: "{}", block_tag: latest)]
            fn capture_cairo0_event() {{
                let cairo0_contract_address = contract_address_const::<0x2c77ca97586968c6651a533bd5f58042c368b14cf5f526d2f42f670012e10ac>();
                let contract = declare("SpyEventsChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ISpyEventsCheckerDispatcher {{ contract_address }};

                let mut spy = spy_events();

                dispatcher.test_cairo0_event_collection(cairo0_contract_address.into());
                dispatcher.emit_one_event(420);
                dispatcher.test_cairo0_event_collection(cairo0_contract_address.into());

                spy.assert_emitted(@array![
                    (
                        cairo0_contract_address,
                        SpyEventsChecker::Event::my_event(
                            SpyEventsChecker::Cairo0Event {{
                                some_data: 123456789
                            }}
                        )
                    ),
                    (
                        contract_address,
                        SpyEventsChecker::Event::FirstEvent(
                            SpyEventsChecker::FirstEvent {{
                                some_data: 420
                            }}
                        )
                    ),
                    (
                        cairo0_contract_address,
                        SpyEventsChecker::Event::my_event(
                            SpyEventsChecker::Cairo0Event {{
                                some_data: 123456789
                            }}
                        )
                    )
                ]);
            }}
        "#,
            node_rpc_url()
        ).as_str(),
        Contract::from_code_path("contract::SpyEventsChecker".to_string(),
            Path::new("tests/data/contracts/spy_events_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn test_filtering() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::ContractAddress;
            use snforge_std::{
                declare, ContractClassTrait, DeclareResultTrait, spy_events, Event,
                EventSpy, EventSpyTrait, EventSpyAssertionsTrait, EventsFilterTrait
            };

            #[starknet::interface]
            trait ISpyEventsChecker<TContractState> {
                fn emit_one_event(ref self: TContractState, some_data: felt252);
            }

            #[starknet::contract]
            mod SpyEventsChecker {
                use starknet::ContractAddress;

                #[storage]
                struct Storage {}

                #[event]
                #[derive(Drop, starknet::Event)]
                enum Event {
                    FirstEvent: FirstEvent
                }

                #[derive(Drop, starknet::Event)]
                struct FirstEvent {
                    some_data: felt252
                }
            }

            #[test]
            fn filter_events() {
                let contract = declare("SpyEventsChecker").unwrap().contract_class();
                let (first_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let (second_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();

                let first_dispatcher = ISpyEventsCheckerDispatcher { contract_address: first_address };
                let second_dispatcher = ISpyEventsCheckerDispatcher { contract_address: second_address };

                let mut spy = spy_events();
                // assert(spy._event_offset == 0, 'Event offset should be 0'); TODO(#2765)

                first_dispatcher.emit_one_event(123);
                second_dispatcher.emit_one_event(234);

                let events_from_first_address = spy.get_events().emitted_by(first_address);
                let events_from_second_address = spy.get_events().emitted_by(second_address);

                let (from, event) = events_from_first_address.events.at(0);
                assert(from == @first_address, 'Emitted from wrong address');
                assert(event.keys.len() == 1, 'There should be one key');
                assert(event.keys.at(0) == @selector!("FirstEvent"), 'Wrong event name');
                assert(event.data.len() == 1, 'There should be one data');

                let (from, event) = events_from_second_address.events.at(0);
                assert(from == @second_address, 'Emitted from wrong address');
                assert(event.keys.len() == 1, 'There should be one key');
                assert(event.keys.at(0) == @selector!("FirstEvent"), 'Wrong event name');
                assert(event.data.len() == 1, 'There should be one data');
            }
        "#,
        ),
        Contract::from_code_path(
            "contract::SpyEventsChecker".to_string(),
            Path::new("tests/data/contracts/spy_events_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn assert_emitted_with_replaced_bytecode() {
    let test = test_case!(
        indoc!(
            r#"
            use core::clone::Clone;
            use snforge_std::{
                declare, replace_bytecode, spy_events, ContractClassTrait, DeclareResultTrait,
                EventSpyAssertionsTrait
            };

            #[starknet::interface]
            trait IReplaceBytecode<TContractState> {
                fn emit_event(ref self: TContractState);
            }

            #[starknet::contract]
            mod ReplaceBytecodeB {
                #[storage]
                struct Storage {}

                #[event]
                #[derive(Drop, starknet::Event)]
                enum Event {
                    ValueChanged: ValueChanged,
                }

                #[derive(Drop, starknet::Event)]
                struct ValueChanged {
                    value: felt252,
                }
            }

            #[test]
            fn assert_emitted_with_replaced_bytecode() {
                let contract = declare("ReplaceBytecodeA").unwrap().contract_class();
                let replacement = declare("ReplaceBytecodeB").unwrap().contract_class().class_hash.clone();
                let (contract_address, _) = contract.deploy(@array![]).unwrap();
                let dispatcher = IReplaceBytecodeDispatcher { contract_address };

                replace_bytecode(contract_address, replacement);

                let mut spy = spy_events();
                dispatcher.emit_event();

                spy.assert_emitted(@array![
                    (
                        contract_address,
                        ReplaceBytecodeB::Event::ValueChanged(
                            ReplaceBytecodeB::ValueChanged { value: 420 }
                        )
                    )
                ]);
            }
        "#
        ),
        Contract::from_code_path(
            "contract::ReplaceBytecodeA",
            Path::new("tests/data/contracts/two_implementations.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "contract::ReplaceBytecodeB",
            Path::new("tests/data/contracts/two_implementations.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn assert_not_emitted_with_replaced_bytecode() {
    let test = test_case!(
        indoc!(
            r#"
            use core::clone::Clone;
            use snforge_std::{
                declare, replace_bytecode, spy_events, ContractClassTrait, DeclareResultTrait,
                EventSpyAssertionsTrait
            };

            #[starknet::interface]
            trait IReplaceBytecode<TContractState> {
                fn emit_event(ref self: TContractState);
            }

            #[starknet::contract]
            mod ReplaceBytecodeB {
                #[storage]
                struct Storage {}

                #[event]
                #[derive(Drop, starknet::Event)]
                enum Event {
                    ValueChanged: ValueChanged,
                }

                #[derive(Drop, starknet::Event)]
                struct ValueChanged {
                    value: felt252,
                }
            }

            #[test]
            fn assert_not_emitted_with_replaced_bytecode() {
                let contract = declare("ReplaceBytecodeA").unwrap().contract_class();
                let replacement = declare("ReplaceBytecodeB").unwrap().contract_class().class_hash.clone();
                let (contract_address, _) = contract.deploy(@array![]).unwrap();
                let dispatcher = IReplaceBytecodeDispatcher { contract_address };

                replace_bytecode(contract_address, replacement);

                let mut spy = spy_events();
                dispatcher.emit_event();

                spy.assert_not_emitted(@array![
                    (
                        contract_address,
                        ReplaceBytecodeB::Event::ValueChanged(
                            ReplaceBytecodeB::ValueChanged { value: 2137 }
                        )
                    )
                ]);
            }
        "#
        ),
        Contract::from_code_path(
            "contract::ReplaceBytecodeA",
            Path::new("tests/data/contracts/two_implementations.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "contract::ReplaceBytecodeB",
            Path::new("tests/data/contracts/two_implementations.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn assert_emitted_with_replaced_bytecode_fails() {
    let test = test_case!(
        indoc!(
            r#"
            use core::clone::Clone;
            use snforge_std::{
                declare, replace_bytecode, spy_events, ContractClassTrait, DeclareResultTrait,
                EventSpyAssertionsTrait
            };

            #[starknet::interface]
            trait IReplaceBytecode<TContractState> {
                fn emit_event(ref self: TContractState);
            }

            #[starknet::contract]
            mod ReplaceBytecodeB {
                #[storage]
                struct Storage {}

                #[event]
                #[derive(Drop, starknet::Event)]
                enum Event {
                    ValueChanged: ValueChanged,
                }

                #[derive(Drop, starknet::Event)]
                struct ValueChanged {
                    value: felt252,
                }
            }

            #[test]
            fn assert_emitted_with_replaced_bytecode_fails() {
                let contract = declare("ReplaceBytecodeA").unwrap().contract_class();
                let replacement = declare("ReplaceBytecodeB").unwrap().contract_class().class_hash.clone();
                let (contract_address, _) = contract.deploy(@array![]).unwrap();
                let dispatcher = IReplaceBytecodeDispatcher { contract_address };

                replace_bytecode(contract_address, replacement);

                let mut spy = spy_events();
                dispatcher.emit_event();

                spy.assert_emitted(@array![
                    (
                        contract_address,
                        ReplaceBytecodeB::Event::ValueChanged(
                            ReplaceBytecodeB::ValueChanged { value: 2137 }
                        )
                    )
                ]);
            }
        "#
        ),
        Contract::from_code_path(
            "contract::ReplaceBytecodeA",
            Path::new("tests/data/contracts/two_implementations.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "contract::ReplaceBytecodeB",
            Path::new("tests/data/contracts/two_implementations.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_failed(&result);
    assert_case_output_contains(
        &result,
        "assert_emitted_with_replaced_bytecode_fails",
        "Event ValueChanged { value: 0x859 } was not emitted from contract::ReplaceBytecodeB at",
    );
}

#[test]
fn assert_not_emitted_with_replaced_bytecode_fails() {
    let test = test_case!(
        indoc!(
            r#"
            use core::clone::Clone;
            use snforge_std::{
                declare, replace_bytecode, spy_events, ContractClassTrait, DeclareResultTrait,
                EventSpyAssertionsTrait
            };

            #[starknet::interface]
            trait IReplaceBytecode<TContractState> {
                fn emit_event(ref self: TContractState);
            }

            #[starknet::contract]
            mod ReplaceBytecodeB {
                #[storage]
                struct Storage {}

                #[event]
                #[derive(Drop, starknet::Event)]
                enum Event {
                    ValueChanged: ValueChanged,
                }

                #[derive(Drop, starknet::Event)]
                struct ValueChanged {
                    value: felt252,
                }
            }

            #[test]
            fn assert_not_emitted_with_replaced_bytecode_fails() {
                let contract = declare("ReplaceBytecodeA").unwrap().contract_class();
                let replacement = declare("ReplaceBytecodeB").unwrap().contract_class().class_hash.clone();
                let (contract_address, _) = contract.deploy(@array![]).unwrap();
                let dispatcher = IReplaceBytecodeDispatcher { contract_address };

                replace_bytecode(contract_address, replacement);

                let mut spy = spy_events();
                dispatcher.emit_event();

                spy.assert_not_emitted(@array![
                    (
                        contract_address,
                        ReplaceBytecodeB::Event::ValueChanged(
                            ReplaceBytecodeB::ValueChanged { value: 420 }
                        )
                    )
                ]);
            }
        "#
        ),
        Contract::from_code_path(
            "contract::ReplaceBytecodeA",
            Path::new("tests/data/contracts/two_implementations.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "contract::ReplaceBytecodeB",
            Path::new("tests/data/contracts/two_implementations.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_failed(&result);
    assert_case_output_contains(
        &result,
        "assert_not_emitted_with_replaced_bytecode_fails",
        "Event ValueChanged { value: 0x1a4 } was emitted from contract::ReplaceBytecodeB at",
    );
}
