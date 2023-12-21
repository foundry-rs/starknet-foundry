use indoc::indoc;
use std::path::Path;
use test_utils::runner::Contract;
use test_utils::running_tests::run_test_case;
use test_utils::{assert_case_output_contains, assert_failed, assert_passed, test_case};

#[test]
fn spy_events_simple() {
    let test = test_case!(
        indoc!(
            r"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::ContractAddress;
            use snforge_std::{ declare, ContractClassTrait, spy_events, EventSpy, EventFetcher,
                event_name_hash, EventAssertions, SpyOn };

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
                let contract = declare('SpyEventsChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

                let mut spy = spy_events(SpyOn::One(contract_address));
                assert(spy._id == 0, 'Id should be 0');

                dispatcher.emit_one_event(123);

                spy.assert_emitted(@array![
                    (
                        contract_address,
                        SpyEventsChecker::Event::FirstEvent(
                            SpyEventsChecker::FirstEvent { some_data: 123 }
                        )
                    )
                ]);
                assert(spy.events.len() == 0, 'There should be no events');
            }
        "
        ),
        Contract::from_code_path(
            "SpyEventsChecker".to_string(),
            Path::new("tests/data/contracts/spy_events_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn assert_emitted_fails() {
    let test = test_case!(
        indoc!(
            r"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::ContractAddress;
            use snforge_std::{ declare, ContractClassTrait, spy_events, EventSpy,
                EventAssertions, SpyOn };

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
                let contract = declare('SpyEventsChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

                let mut spy = spy_events(SpyOn::One(contract_address));
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
        "
        ),
        Contract::from_code_path(
            "SpyEventsChecker".to_string(),
            Path::new("tests/data/contracts/spy_events_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_failed!(result);
    assert_case_output_contains!(
        result,
        "assert_emitted_fails",
        "Event with matching data and"
    );
    assert_case_output_contains!(result, "assert_emitted_fails", "keys was not emitted");
}

#[test]
fn expect_three_events_while_two_emitted() {
    let test = test_case!(
        indoc!(
            r"
            use array::ArrayTrait;
            use result::ResultTrait;
            use traits::Into;
            use starknet::contract_address_const;
            use starknet::ContractAddress;
            use snforge_std::{ declare, ContractClassTrait, spy_events, EventSpy,
                EventAssertions, SpyOn };

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
                let contract = declare('SpyEventsChecker');
                let contract_address = contract.deploy(@array![]).unwrap();
                let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

                let some_data = 456;
                let some_more_data = contract_address_const::<789>();
                let even_more_data = 0;

                let mut spy = spy_events(SpyOn::One(contract_address));
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
        "
        ),
        Contract::from_code_path(
            "SpyEventsChecker".to_string(),
            Path::new("tests/data/contracts/spy_events_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_failed!(result);
    assert_case_output_contains!(
        result,
        "expect_three_events_while_two_emitted",
        "Event with matching data and"
    );
    assert_case_output_contains!(
        result,
        "expect_three_events_while_two_emitted",
        "keys was not emitted"
    );
}

#[test]
fn expect_two_events_while_three_emitted() {
    let test = test_case!(
        indoc!(
            r"
            use array::ArrayTrait;
            use result::ResultTrait;
            use traits::Into;
            use starknet::contract_address_const;
            use starknet::ContractAddress;
            use snforge_std::{ declare, ContractClassTrait, spy_events, EventSpy,
                EventAssertions, SpyOn };

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
                let contract = declare('SpyEventsChecker');
                let contract_address = contract.deploy(@array![]).unwrap();
                let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

                let some_data = 456;
                let some_more_data = contract_address_const::<789>();
                let even_more_data = u256 { low: 1, high: 0 };

                let mut spy = spy_events(SpyOn::One(contract_address));
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
        "
        ),
        Contract::from_code_path(
            "SpyEventsChecker".to_string(),
            Path::new("tests/data/contracts/spy_events_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn event_emitted_wrong_data_asserted() {
    let test = test_case!(
        indoc!(
            r"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::ContractAddress;
            use snforge_std::{ declare, ContractClassTrait, spy_events, EventSpy,
                EventAssertions, SpyOn };

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
                let contract = declare('SpyEventsChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

                let mut spy = spy_events(SpyOn::One(contract_address));
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
        "
        ),
        Contract::from_code_path(
            "SpyEventsChecker".to_string(),
            Path::new("tests/data/contracts/spy_events_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_failed!(result);
    assert_case_output_contains!(
        result,
        "event_emitted_wrong_data_asserted",
        "Event with matching data and"
    );
    assert_case_output_contains!(
        result,
        "event_emitted_wrong_data_asserted",
        "keys was not emitted from"
    );
    assert_case_output_contains!(
        result,
        "event_emitted_wrong_data_asserted",
        "2004704341135420872438152444339940174968271850429444763384575162285907940280"
    );
}

#[test]
fn emit_unnamed_event() {
    let test = test_case!(
        indoc!(
            r"
            use array::ArrayTrait;
            use result::ResultTrait;
            use traits::Into;
            use starknet::contract_address_const;
            use starknet::ContractAddress;

            use snforge_std::{ declare, ContractClassTrait, spy_events, EventSpy, EventFetcher,
                event_name_hash, EventAssertions, Event, SpyOn };

            #[starknet::interface]
            trait ISpyEventsChecker<TContractState> {
                fn emit_event_syscall(ref self: TContractState, some_key: felt252, some_data: felt252);
            }

            #[test]
            fn emit_unnamed_event() {
                let contract = declare('SpyEventsChecker');
                let contract_address = contract.deploy(@array![]).unwrap();
                let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

                let mut spy = spy_events(SpyOn::One(contract_address));
                dispatcher.emit_event_syscall(123, 456);

                spy.assert_emitted(@array![
                    (
                        contract_address,
                        Event { keys: array![123], data: array![456] }
                    )
                ]);
            }
        "
        ),
        Contract::from_code_path(
            "SpyEventsChecker".to_string(),
            Path::new("tests/data/contracts/spy_events_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn assert_not_emitted_pass() {
    let test = test_case!(
        indoc!(
            r"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::ContractAddress;
            use snforge_std::{ declare, ContractClassTrait, spy_events, EventSpy,
                EventAssertions, SpyOn };

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
                let contract = declare('SpyEventsChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

                let mut spy = spy_events(SpyOn::One(contract_address));
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
        "
        ),
        Contract::from_code_path(
            "SpyEventsChecker".to_string(),
            Path::new("tests/data/contracts/spy_events_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn assert_not_emitted_fails() {
    let test = test_case!(
        indoc!(
            r"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::ContractAddress;
            use snforge_std::{ declare, ContractClassTrait, spy_events, EventSpy, EventFetcher,
                event_name_hash, EventAssertions, SpyOn };

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
                let contract = declare('SpyEventsChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

                let mut spy = spy_events(SpyOn::One(contract_address));
                assert(spy._id == 0, 'Id should be 0');

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
        "
        ),
        Contract::from_code_path(
            "SpyEventsChecker".to_string(),
            Path::new("tests/data/contracts/spy_events_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_failed!(result);
    assert_case_output_contains!(
        result,
        "assert_not_emitted_fails",
        "Event with matching data and"
    );
    assert_case_output_contains!(result, "assert_not_emitted_fails", "keys was emitted");
}
