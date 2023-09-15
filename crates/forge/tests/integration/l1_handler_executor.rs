use crate::integration::common::runner::Contract;
use crate::integration::common::running_tests::run_test_case;
use crate::{assert_passed, test_case};
use indoc::indoc;
use std::path::Path;

#[test]
fn l1_handler_executor() {
    let test = test_case!(
        indoc!(
            r#"

            #[derive(Copy, Serde, Drop)]
            struct L1Data {
                balance: felt252,
                token_id: u256
            }

            #[starknet::interface]
            trait IBalanceToken<TContractState> {
                fn get_balance(self: @TContractState) -> felt252;
                fn get_token_id(self: @TContractState) -> u256;
            }

            use serde::Serde;
            use array::{ArrayTrait, SpanTrait};
            use core::result::ResultTrait;
            use snforge_std::{declare, ContractClassTrait, L1Handler, L1HandlerTrait};

            #[test]
            fn test_l1_handler_execute() {
                let calldata = array![0x123];

                let contract = declare('l1_handler_executor');
                let contract_address = contract.deploy(@calldata).unwrap();

                let l1_data = L1Data {
                    balance: 42,
                    token_id: 8888_u256,
                };

                let mut payload: Array<felt252> = ArrayTrait::new();
                l1_data.serialize(ref payload);

                let mut l1_handler = L1HandlerTrait::new(
                    contract_address,
                    function_name: 'process_l1_message'
                );

                l1_handler.from_address = 0x123;
                l1_handler.payload = payload.span();

                l1_handler.execute();

                let dispatcher = IBalanceTokenDispatcher { contract_address };
                assert(dispatcher.get_balance() == 42, 'Invalid balance');
                assert(dispatcher.get_token_id() == 8888_u256, 'Invalid token id');
            }
        "#
        ),
        Contract::from_code_path(
            "l1_handler_executor".to_string(),
            Path::new("tests/data/contracts/l1_handler_execute_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}
