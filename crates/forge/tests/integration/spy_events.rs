use crate::integration::common::corelib::{corelib_path, predeployed_contracts};
use crate::integration::common::runner::Contract;
use crate::{assert_failed, assert_passed, test_case};
use camino::Utf8PathBuf;
use forge::run;
use indoc::indoc;
use std::path::Path;
use crate::integration::common::running_tests::run_test_case;

#[test]
fn spy_events_complex() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::ContractAddress;
            use snforge_std::{ declare, ContractClassTrait, spy_events, EventSpy,
                EventFetcher, event_name_hash };

            #[starknet::interface]
            trait IEventEmitter<TContractState> {
                fn emit_one_event(ref self: TContractState, some_data: felt252);
            }

            #[test]
            fn test_expect_events_complex() {
                let contract = declare('EventEmitter');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IEventEmitterDispatcher { contract_address };

                let mut spy = spy_events();
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
            "EventEmitter".to_string(),
            Path::new("tests/data/contracts/event_emitter.cairo"),
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
                event_name_hash, EventAssertions, Event };

            #[starknet::interface]
            trait IEventEmitter<TContractState> {
                fn emit_one_event(ref self: TContractState, some_data: felt252);
            }

            #[test]
            fn test_expect_events_simple() {
                let contract = declare('EventEmitter');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IEventEmitterDispatcher { contract_address };

                let mut spy = spy_events();
                dispatcher.emit_one_event(123);

                spy.assert_emitted(@array![
                    Event { from: contract_address, name: 'FirstEvent', keys: array![], data: array![] }
                ])
            }
        "#
        ),
        Contract::from_code_path(
            "EventEmitter".to_string(),
            Path::new("tests/data/contracts/event_emitter.cairo"),
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
                event_name_hash, EventAssertions, Event };

            #[starknet::interface]
            trait IEventEmitter<TContractState> {
                fn do_not_emit(ref self: TContractState);
            }

            #[test]
            fn test_expect_events_simple() {
                let contract = declare('EventEmitter');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IEventEmitterDispatcher { contract_address };

                let mut spy = spy_events();
                dispatcher.do_not_emit();

                spy.assert_emitted(@array![
                    Event { from: contract_address, name: 'FirstEvent', keys: array![], data: array![] }
                ]);
            }
        "#
        ),
        Contract::from_code_path(
            "EventEmitter".to_string(),
            Path::new("tests/data/contracts/event_emitter.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_failed!(result);
}
