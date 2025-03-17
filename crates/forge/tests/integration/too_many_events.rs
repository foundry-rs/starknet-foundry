use blockifier::versioned_constants::{EventLimits, VersionedConstants};
use indoc::formatdoc;
use std::path::Path;
use test_utils::runner::{Contract, assert_case_output_contains, assert_failed, assert_passed};
use test_utils::running_tests::run_test_case;

#[test]
fn ok_events() {
    let EventLimits {
        max_data_length,
        max_keys_length,
        max_n_emitted_events,
    } = VersionedConstants::latest_constants().tx_event_limits;

    let test = test_utils::test_case!(
        &formatdoc!(
            r#"
            use starknet::ContractAddress;
            use snforge_std::{{ declare, ContractClassTrait, DeclareResultTrait, store, load }};

            #[starknet::interface]
            trait ITooManyEvents<TContractState> {{
                fn emit_too_many_events(self: @TContractState, count: felt252);
                fn emit_too_many_keys(self: @TContractState, count: felt252);
                fn emit_too_many_data(self: @TContractState, count: felt252);
            }}

            fn deploy_contract() -> ITooManyEventsDispatcher {{
                let contract = declare("TooManyEvents").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@array![]).unwrap();
                ITooManyEventsDispatcher {{ contract_address }}
            }}

            #[test]
            fn emit_ok_many_events() {{
                let deployed = deploy_contract();

                deployed.emit_too_many_events({max_n_emitted_events});

                assert(1 == 1, '');
            }}
            #[test]
            fn emit_ok_many_keys() {{
                let deployed = deploy_contract();

                deployed.emit_too_many_keys({max_keys_length});

                assert(1 == 1, '');
            }}
            #[test]
            fn emit_ok_many_data() {{
                let deployed = deploy_contract();

                deployed.emit_too_many_data({max_data_length});

                assert(1 == 1, '');
            }}
        "#
        ),
        Contract::from_code_path(
            "TooManyEvents".to_string(),
            Path::new("tests/data/contracts/too_many_events.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}
#[test]
fn too_many_events() {
    let EventLimits {
        max_data_length,
        max_keys_length,
        max_n_emitted_events,
    } = VersionedConstants::latest_constants().tx_event_limits;

    let emit_too_many_events = max_n_emitted_events + 1;
    let emit_too_many_keys = max_keys_length + 1;
    let emit_too_many_data = max_data_length + 1;

    let test = test_utils::test_case!(
        &formatdoc!(
            r#"
            use starknet::ContractAddress;
            use snforge_std::{{ declare, ContractClassTrait, DeclareResultTrait, store, load }};

            #[starknet::interface]
            trait ITooManyEvents<TContractState> {{
                fn emit_too_many_events(self: @TContractState, count: felt252);
                fn emit_too_many_keys(self: @TContractState, count: felt252);
                fn emit_too_many_data(self: @TContractState, count: felt252);
            }}

            fn deploy_contract() -> ITooManyEventsDispatcher {{
                let contract = declare("TooManyEvents").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@array![]).unwrap();
                ITooManyEventsDispatcher {{ contract_address }}
            }}

            #[test]
            fn emit_too_many_events() {{
                let deployed = deploy_contract();

                deployed.emit_too_many_events({emit_too_many_events});

                assert(1 == 1, '');
            }}
            #[test]
            fn emit_too_many_keys() {{
                let deployed = deploy_contract();

                deployed.emit_too_many_keys({emit_too_many_keys});

                assert(1 == 1, '');
            }}
            #[test]
            fn emit_too_many_data() {{
                let deployed = deploy_contract();

                deployed.emit_too_many_data({emit_too_many_data});

                assert(1 == 1, '');
            }}
        "#
        ),
        Contract::from_code_path(
            "TooManyEvents".to_string(),
            Path::new("tests/data/contracts/too_many_events.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_failed(&result);
    assert_case_output_contains(
        &result,
        "emit_too_many_events",
        &format!(
            "Got an exception while executing a hint: Exceeded the maximum number of events, number events: {emit_too_many_events}, max number events: {max_n_emitted_events}."
        ),
    );
    assert_case_output_contains(
        &result,
        "emit_too_many_data",
        &format!(
            "Got an exception while executing a hint: Exceeded the maximum data length, data length: {emit_too_many_data}, max data length: {max_data_length}."
        ),
    );
    assert_case_output_contains(
        &result,
        "emit_too_many_keys",
        &format!(
            "Got an exception while executing a hint: Exceeded the maximum keys length, keys length: {emit_too_many_keys}, max keys length: {max_keys_length}."
        ),
    );
}
