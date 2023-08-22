use crate::integration::common::runner::Contract;
use crate::integration::common::running_tests::run_test_case;
use crate::{assert_case_output_contains, assert_failed, assert_passed, test_case};
use indoc::indoc;
use std::path::Path;

#[test]
fn spy_events_complex() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::ContractAddress;
            use snforge_std::{ declare, ContractClassTrait, spy_events, EventSpy,
                EventFetcher, event_name_hash, SpyOn };

            #[starknet::interface]
            trait ISpyEventsChecker<TContractState> {
                fn emit_one_event(ref self: TContractState, some_data: felt252);
            }

            #[test]
            fn test_expect_events_complex() {
                let contract = declare('SpyEventsChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ISpyEventsCheckerDispatcher { contract_address };

                let mut spy = spy_events(SpyOn::All);
                dispatcher.emit_one_event(123);
                spy.fetch_events();

                assert(spy.events.len() == 1, 'There should be one event');
                assert(spy.events.at(0).name == @event_name_hash('FirstEvent'), 'Wrong event name');
                assert(spy.events.at(0).keys.len() == 0, 'There should be no keys');
                assert(spy.events.at(0).data.len() == 1, 'There should be one data');

                dispatcher.emit_one_event(123);
                assert(spy.events.len() == 1, 'There should be one event');

                spy.fetch_events();
                assert(spy.events.len() == 2, 'There should be two events');
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
    assert_case_output_contains!(
        result,
        "test_spy_on_multiple_contracts",
        "FirstEvent"
    );
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
    assert_case_output_contains!(
        result,
        "test_assert_wrong_data",
        "FirstEvent"
    );
    assert_case_output_contains!(
        result,
        "test_assert_wrong_data",
        "event was emitted from"
    );
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
