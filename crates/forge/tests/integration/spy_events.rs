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
            use snforge_std::{ declare, PreparedContract, deploy, spy_events, EventSpy, EventFetcher, EventFetcherImpl };

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

                let mut spy = spy_events();
                dispatcher.emit_one_event(123);
                spy.fetch_events();

                assert(spy.events.len() == 1, 'There should be one event');
                assert(spy.events.at(0).keys.len() == 0, 'There should be no keys');

                let data: Array<felt252> = array![123];
                let mut i = 0;
                loop {
                    if i >= data.len() {
                        break;
                    }
                    assert(spy.events.at(0).data.at(i) == data.at(i), 'Event data is invalid');
                    i += 1;
                }
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
