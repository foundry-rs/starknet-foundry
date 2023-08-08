use crate::integration::common::corelib::{corelib_path, predeployed_contracts};
use crate::integration::common::runner::Contract;
use crate::{assert_failed, assert_passed, test_case};
use camino::Utf8PathBuf;
use forge::run;
use indoc::indoc;
use std::path::Path;

#[test]
fn expect_events_simple() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::ContractAddress;
            use cheatcodes::{ declare, PreparedContract, deploy, expect_events, Event };

            #[starknet::interface]
            trait IEventEmitter<TContractState> {
                fn emit_one_event(ref self: TContractState, some_data: felt252);
            }

            #[test]
            fn test_expect_events_simple() {
                let class_hash = declare('EventEmitter');
                let prepared = PreparedContract { class_hash, constructor_calldata: @ArrayTrait::new() };
                let contract_address = deploy(prepared).unwrap();
                let dispatcher = IEventEmitterDispatcher { contract_address };

                expect_events(array![Event { name: 'FirstEvent', keys: array![], data: array![123] }]);
                dispatcher.emit_one_event(123);
            }
        "#
        ),
        Contract::from_code_path(
            "EventEmitter".to_string(),
            Path::new("tests/data/contracts/event_emitter.cairo"),
        )
        .unwrap()
    );

    let result = run(
        &test.path().unwrap(),
        &String::from("src"),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();

    assert_passed!(result);
}

#[test]
fn expect_events_fails() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::ContractAddress;
            use cheatcodes::{ declare, PreparedContract, deploy, expect_events, Event };

            #[starknet::interface]
            trait IEventEmitter<TContractState> {
                fn do_not_emit(ref self: TContractState);
            }

            #[test]
            fn test_expect_events_fails() {
                let class_hash = declare('EventEmitter');
                let prepared = PreparedContract { class_hash, constructor_calldata: @ArrayTrait::new() };
                let contract_address = deploy(prepared).unwrap();
                let dispatcher = IEventEmitterDispatcher { contract_address };

                expect_events(array![Event { name: 'FirstEvent', keys: array![], data: array![123] }]);
                dispatcher.do_not_emit();
            }
        "#
        ),
        Contract::from_code_path(
            "EventEmitter".to_string(),
            Path::new("tests/data/contracts/event_emitter.cairo"),
        )
        .unwrap()
    );

    let result = run(
        &test.path().unwrap(),
        &String::from("src"),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();

    assert_failed!(result);
}

#[test]
fn expect_events_complex() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::ContractAddress;
            use starknet::contract_address_const;
            use cheatcodes::{ declare, PreparedContract, deploy, expect_events, Event };

            #[starknet::interface]
            trait IEventEmitter<TContractState> {
                fn do_not_emit(ref self: TContractState);
                fn emit_one_event(ref self: TContractState, some_data: felt252);
                fn emit_two_events(ref self: TContractState, some_data: felt252, some_more_data: ContractAddress);
                fn emit_three_events(ref self: TContractState, some_data: felt252, some_more_data: ContractAddress, even_more_data: u256);
            }

            #[test]
            fn test_expect_events_simple() {
                let class_hash = declare('EventEmitter');
                let prepared = PreparedContract { class_hash, constructor_calldata: @ArrayTrait::new() };
                let contract_address = deploy(prepared).unwrap();
                let dispatcher = IEventEmitterDispatcher { contract_address };

                expect_events(array![Event { name: 'FirstEvent', keys: array![], data: array![123] }]);
                dispatcher.emit_one_event(123);

                dispatcher.do_not_emit();

                expect_events(array![
                    Event { name: 'FirstEvent', keys: array![], data: array![456] },
                    Event { name: 'SecondEvent', keys: array![789], data: array![456] },
                    Event { name: 'ThirdEvent', keys: array![], data: array![456, 789, 1, 1] },
                ]);
                dispatcher.emit_three_events(456, contract_address_const::<789>(), u256 { low: 1, high: 1 });

                dispatcher.emit_two_events(456, contract_address_const::<789>());
            }
        "#
        ),
        Contract::from_code_path(
            "EventEmitter".to_string(),
            Path::new("tests/data/contracts/event_emitter.cairo"),
        )
        .unwrap()
    );

    let result = run(
        &test.path().unwrap(),
        &String::from("src"),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();

    assert_passed!(result);
}

#[test]
fn expect_two_from_three_events() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::ContractAddress;
            use starknet::contract_address_const;
            use cheatcodes::{ declare, PreparedContract, deploy, expect_events, Event };

            #[starknet::interface]
            trait IEventEmitter<TContractState> {
                fn emit_three_events(ref self: TContractState, some_data: felt252, some_more_data: ContractAddress, even_more_data: u256);
            }

            #[test]
            fn test_expect_events_simple() {
                let class_hash = declare('EventEmitter');
                let prepared = PreparedContract { class_hash, constructor_calldata: @ArrayTrait::new() };
                let contract_address = deploy(prepared).unwrap();
                let dispatcher = IEventEmitterDispatcher { contract_address };

                expect_events(array![
                    Event { name: 'FirstEvent', keys: array![], data: array![456] },
                    Event { name: 'ThirdEvent', keys: array![], data: array![456, 789, 1, 1] },
                ]);
                dispatcher.emit_three_events(456, contract_address_const::<789>(), u256 { low: 1, high: 1 });
            }
        "#
        ),
        Contract::from_code_path(
            "EventEmitter".to_string(),
            Path::new("tests/data/contracts/event_emitter.cairo"),
        )
        .unwrap()
    );

    let result = run(
        &test.path().unwrap(),
        &String::from("src"),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();

    assert_passed!(result);
}

#[test]
fn expect_three_events_while_two_emitted() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::ContractAddress;
            use starknet::contract_address_const;
            use cheatcodes::{ declare, PreparedContract, deploy, expect_events, Event };

            #[starknet::interface]
            trait IEventEmitter<TContractState> {
                fn emit_two_events(ref self: TContractState, some_data: felt252, some_more_data: ContractAddress);
            }

            #[test]
            fn test_expect_events_simple() {
                let class_hash = declare('EventEmitter');
                let prepared = PreparedContract { class_hash, constructor_calldata: @ArrayTrait::new() };
                let contract_address = deploy(prepared).unwrap();
                let dispatcher = IEventEmitterDispatcher { contract_address };

                expect_events(array![
                    Event { name: 'FirstEvent', keys: array![], data: array![456] },
                    Event { name: 'SecondEvent', keys: array![789], data: array![456] },
                    Event { name: 'ThirdEvent', keys: array![], data: array![456, 789, 1, 1] },
                ]);
                dispatcher.emit_two_events(456, contract_address_const::<789>());
            }
        "#
        ),
        Contract::from_code_path(
            "EventEmitter".to_string(),
            Path::new("tests/data/contracts/event_emitter.cairo"),
        )
        .unwrap()
    );

    let result = run(
        &test.path().unwrap(),
        &String::from("src"),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();

    assert_failed!(result);
}
