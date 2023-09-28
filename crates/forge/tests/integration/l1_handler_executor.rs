use indoc::indoc;
use std::path::Path;
use utils::runner::Contract;
use utils::running_tests::run_test_case;
use utils::{assert_passed, test_case};

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
            use snforge_std::{declare, ContractClassTrait, L1Handler, L1HandlerTrait, RevertedTransaction};

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

                l1_handler.execute().unwrap();
                    
                let dispatcher = IBalanceTokenDispatcher { contract_address };
                assert(dispatcher.get_balance() == 42, dispatcher.get_balance());
                assert(dispatcher.get_token_id() == 8888_u256, 'Invalid token id');
            }
             
            #[test]
            fn test_l1_handler_execute_panicking() {
                let calldata = array![0x123];

                let contract = declare('l1_handler_executor');
                let contract_address = contract.deploy(@calldata).unwrap();


                let mut l1_handler = L1HandlerTrait::new(
                    contract_address,
                    function_name: 'panicking_l1_handler'
                );

                l1_handler.from_address = 0x123;
                l1_handler.payload = array![].span();
                match l1_handler.execute() {
                    Result::Ok(_) => panic_with_felt252('should have panicked'),
                    Result::Err(RevertedTransaction { panic_data }) => {
                        assert(*panic_data.at(0) == 'custom', 'Wrong 1st panic datum');
                        assert(*panic_data.at(1) == 'panic', 'Wrong 2nd panic datum');
                    },
                }
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
