use crate::integration::common::runner::Contract;
use crate::integration::common::running_tests::run_test_case;
use crate::{assert_case_output_contains, assert_failed, assert_passed, test_case};
use indoc::indoc;
use std::path::Path;

#[test]
fn spy_events_simple() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::ContractAddress;
            use snforge_std::{ declare, ContractClassTrait, spy_events, EventSpy, EventFetcher,
                event_name_hash, EventAssertions, Event, SpyOn };

            #[starknet::interface]
            trait ISpyEventsChecker<TContractState> {
                fn emit_one_event(ref self: TContractState, some_data: felt252);
            }

            #[test]
            fn test_expect_events_simple() {
                let contract = declare('SpyEventsChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

                let mut spy = spy_events(SpyOn::One(contract_address));
                assert(spy._id == 0, 'Id should be 0');

                dispatcher.emit_one_event(123);

                spy.assert_emitted(@array![
                    Event { from: contract_address, name: 'FirstEvent', keys: array![], data: array![123] }
                ]);
                assert(spy.events.len() == 0, 'There should be no events');
            }
        "#
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
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::ContractAddress;
            use snforge_std::{ declare, ContractClassTrait, spy_events, EventSpy, EventFetcher,
                event_name_hash, EventAssertions, Event, SpyOn };

            #[starknet::interface]
            trait ISpyEventsChecker<TContractState> {
                fn do_not_emit(ref self: TContractState);
            }

            #[test]
            fn test_expect_events_simple() {
                let contract = declare('SpyEventsChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

                let mut spy = spy_events(SpyOn::One(contract_address));
                dispatcher.do_not_emit();

                spy.assert_emitted(@array![
                    Event { from: contract_address, name: 'FirstEvent', keys: array![], data: array![] }
                ]);
            }
        "#
        ),
        Contract::from_code_path(
            "SpyEventsChecker".to_string(),
            Path::new("tests/data/contracts/spy_events_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_failed!(result);
    assert_case_output_contains!(result, "test_expect_events_simple", "FirstEvent");
    assert_case_output_contains!(result, "test_expect_events_simple", "event was not emitted");
}

#[test]
fn test_nested_calls() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use traits::Into;
            use starknet::ContractAddress;
            use snforge_std::{ declare, ContractClassTrait, spy_events, EventSpy, EventFetcher,
                event_name_hash, EventAssertions, Event, SpyOn };

            #[starknet::interface]
            trait ISpyEventsChecker<TContractState> {
                fn emit_one_event(ref self: TContractState, some_data: felt252);
            }

            #[test]
            fn test_nested_calls() {
                let spy_events_checker = declare('SpyEventsChecker');
                let spy_events_checker_address = spy_events_checker.deploy(@array![]).unwrap();

                let spy_events_checker_proxy = declare('SpyEventsCheckerProxy');
                let spy_events_checker_proxy_address =
                    spy_events_checker_proxy.deploy(@array![spy_events_checker_address.into()]).unwrap();

                let spy_events_checker_top_proxy_address =
                    spy_events_checker_proxy.deploy(@array![spy_events_checker_proxy_address.into()]).unwrap();

                let dispatcher = ISpyEventsCheckerDispatcher { contract_address: spy_events_checker_top_proxy_address };

                let mut spy = spy_events(SpyOn::All);
                dispatcher.emit_one_event(222);

                spy.assert_emitted(@array![
                    Event { from: spy_events_checker_address, name: 'FirstEvent', keys: array![], data: array![222] },
                    Event { from: spy_events_checker_proxy_address, name: 'FirstEvent', keys: array![], data: array![222] },
                    Event { from: spy_events_checker_top_proxy_address, name: 'FirstEvent', keys: array![], data: array![222] }
                ]);
            }
        "#
        ),
        Contract::from_code_path(
            "SpyEventsChecker".to_string(),
            Path::new("tests/data/contracts/spy_events_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "SpyEventsCheckerProxy".to_string(),
            Path::new("tests/data/contracts/spy_events_checker_proxy.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
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
            use snforge_std::{ declare, ContractClassTrait, spy_events, EventSpy, EventFetcher,
                event_name_hash, EventAssertions, Event, SpyOn };

            #[starknet::interface]
            trait ISpyEventsChecker<TContractState> {
                fn emit_two_events(ref self: TContractState, some_data: felt252, some_more_data: ContractAddress);
            }

            #[test]
            fn test_expect_three_events_while_two_emitted() {
                let contract = declare('SpyEventsChecker');
                let contract_address = contract.deploy(@array![]).unwrap();
                let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

                let mut spy = spy_events(SpyOn::One(contract_address));
                dispatcher.emit_two_events(456, contract_address_const::<789>());

                spy.assert_emitted(@array![
                    Event { from: contract_address, name: 'FirstEvent', keys: array![], data: array![456] },
                    Event { from: contract_address, name: 'SecondEvent', keys: array![789], data: array![456] },
                    Event { from: contract_address, name: 'ThirdEvent', keys: array![], data: array![] },
                ]);
            }
        "#
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
        "test_expect_three_events_while_two_emitted",
        "ThirdEvent"
    );
    assert_case_output_contains!(
        result,
        "test_expect_three_events_while_two_emitted",
        "event was not emitted"
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
            use snforge_std::{ declare, ContractClassTrait, spy_events, EventSpy, EventFetcher,
                event_name_hash, EventAssertions, Event, SpyOn };

            #[starknet::interface]
            trait ISpyEventsChecker<TContractState> {
                fn emit_three_events(
                    ref self: TContractState,
                    some_data: felt252,
                    some_more_data: ContractAddress,
                    even_more_data: u256
                );
            }

            #[test]
            fn test_expect_three_events_while_two_emitted() {
                let contract = declare('SpyEventsChecker');
                let contract_address = contract.deploy(@array![]).unwrap();
                let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

                let mut spy = spy_events(SpyOn::One(contract_address));
                dispatcher.emit_three_events(456, contract_address_const::<789>(), u256 { low: 1, high: 0 });

                spy.assert_emitted(@array![
                    Event { from: contract_address, name: 'FirstEvent', keys: array![], data: array![456] },
                    Event { from: contract_address, name: 'ThirdEvent', keys: array![], data: array![456, 789, 1, 0] },
                ]);
            }
        "#
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
fn spy_on_multiple_contracts() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use traits::Into;
            use starknet::ContractAddress;
            use snforge_std::{ declare, ContractClassTrait, spy_events, EventSpy, EventFetcher,
                event_name_hash, EventAssertions, Event, SpyOn };

            #[starknet::interface]
            trait ISpyEventsChecker<TContractState> {
                fn emit_one_event(ref self: TContractState, some_data: felt252);
            }

            #[test]
            fn test_spy_on_multiple_contracts() {
                let spy_events_checker = declare('SpyEventsChecker');
                let spy_events_checker_address = spy_events_checker.deploy(@array![]).unwrap();

                let spy_events_checker_proxy = declare('SpyEventsCheckerProxy');
                let spy_events_checker_proxy_address =
                    spy_events_checker_proxy.deploy(@array![spy_events_checker_address.into()]).unwrap();

                let spy_events_checker_top_proxy_address =
                    spy_events_checker_proxy.deploy(@array![spy_events_checker_proxy_address.into()]).unwrap();

                let dispatcher = ISpyEventsCheckerDispatcher { contract_address: spy_events_checker_top_proxy_address };

                let mut spy = spy_events(SpyOn::Multiple(array![spy_events_checker_address, spy_events_checker_proxy_address]));
                dispatcher.emit_one_event(222);

                spy.assert_emitted(@array![
                    Event { from: spy_events_checker_address, name: 'FirstEvent', keys: array![], data: array![222] },
                    Event { from: spy_events_checker_proxy_address, name: 'FirstEvent', keys: array![], data: array![222] },
                    Event { from: spy_events_checker_top_proxy_address, name: 'FirstEvent', keys: array![], data: array![222] }
                ]);
            }
        "#
        ),
        Contract::from_code_path(
            "SpyEventsChecker".to_string(),
            Path::new("tests/data/contracts/spy_events_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "SpyEventsCheckerProxy".to_string(),
            Path::new("tests/data/contracts/spy_events_checker_proxy.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_failed!(result);
    assert_case_output_contains!(result, "test_spy_on_multiple_contracts", "FirstEvent");
    assert_case_output_contains!(
        result,
        "test_spy_on_multiple_contracts",
        "event was not emitted from"
    );
    assert_case_output_contains!(
        result,
        "test_spy_on_multiple_contracts",
        "2851901425299487987270770980201168426168489605512291031522895630582264352821"
    );
}

#[test]
fn event_emitted_wrong_data_asserted() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::ContractAddress;
            use snforge_std::{ declare, ContractClassTrait, spy_events, EventSpy, EventFetcher,
                event_name_hash, EventAssertions, Event, SpyOn };

            #[starknet::interface]
            trait ISpyEventsChecker<TContractState> {
                fn emit_one_event(ref self: TContractState, some_data: felt252);
            }

            #[test]
            fn test_assert_wrong_data() {
                let contract = declare('SpyEventsChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

                let mut spy = spy_events(SpyOn::One(contract_address));
                dispatcher.emit_one_event(123);

                spy.assert_emitted(@array![
                    Event { from: contract_address, name: 'FirstEvent', keys: array![], data: array![124] }
                ]);
            }
        "#
        ),
        Contract::from_code_path(
            "SpyEventsChecker".to_string(),
            Path::new("tests/data/contracts/spy_events_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_failed!(result);
    assert_case_output_contains!(result, "test_assert_wrong_data", "FirstEvent");
    assert_case_output_contains!(result, "test_assert_wrong_data", "event was emitted from");
    assert_case_output_contains!(
        result,
        "test_assert_wrong_data",
        "313030104761018700624599948735635264152311475736681672041458379825603828958"
    );
    assert_case_output_contains!(
        result,
        "test_assert_wrong_data",
        "but keys or data are different"
    );
}

#[test]
fn use_multiple_spies() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use traits::Into;
            use starknet::ContractAddress;
            use snforge_std::{ declare, ContractClassTrait, spy_events, EventSpy, EventFetcher,
                event_name_hash, EventAssertions, Event, SpyOn };

            #[starknet::interface]
            trait ISpyEventsChecker<TContractState> {
                fn emit_one_event(ref self: TContractState, some_data: felt252);
            }

            #[test]
            fn test_spy_on_multiple_contracts() {
                let events_checker = declare('SpyEventsChecker');
                let events_checker_address = events_checker.deploy(@array![]).unwrap();

                let events_checker_proxy = declare('SpyEventsCheckerProxy');
                let events_checker_proxy_address =
                    events_checker_proxy.deploy(@array![events_checker_address.into()]).unwrap();

                let events_checker_top_proxy_address =
                    events_checker_proxy.deploy(@array![events_checker_proxy_address.into()]).unwrap();

                let dispatcher = ISpyEventsCheckerDispatcher { contract_address: events_checker_top_proxy_address };

                let mut events_checker_spy = spy_events(SpyOn::One(events_checker_address));
                let mut events_checker_proxy_spy = spy_events(SpyOn::One(events_checker_proxy_address));
                let mut events_checker_top_proxy_spy = spy_events(SpyOn::One(events_checker_top_proxy_address));

                dispatcher.emit_one_event(222);

                events_checker_spy.fetch_events();
                events_checker_proxy_spy.fetch_events();
                events_checker_top_proxy_spy.fetch_events();

                events_checker_spy.assert_emitted(@array![
                    Event { from: events_checker_address, name: 'FirstEvent', keys: array![], data: array![222] },
                ]);
                events_checker_proxy_spy.assert_emitted(@array![
                    Event { from: events_checker_proxy_address, name: 'FirstEvent', keys: array![], data: array![222] },
                ]);
                events_checker_top_proxy_spy.assert_emitted(@array![
                    Event { from: events_checker_top_proxy_address, name: 'FirstEvent', keys: array![], data: array![222] }
                ]);

            }
        "#
        ),
        Contract::from_code_path(
            "SpyEventsChecker".to_string(),
            Path::new("tests/data/contracts/spy_events_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "SpyEventsCheckerProxy".to_string(),
            Path::new("tests/data/contracts/spy_events_checker_proxy.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}
