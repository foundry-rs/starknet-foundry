use indoc::indoc;
use std::path::Path;
use test_utils::runner::{assert_passed, Contract};
use test_utils::running_tests::run_test_case; 
use test_utils::test_case;

#[test]
fn l1_handler_execute() {
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
            use snforge_std::errors::{ SyscallResultStringErrorTrait, PanicDataOrString };
            use starknet::{ContractAddress, contract_address_const};

            #[test]
            fn l1_handler_execute() {
                let calldata = array![0x123];

                let contract = declare("l1_handler_executor").unwrap();
                let (contract_address, _) = contract.deploy(@calldata).unwrap();

                let l1_data = L1Data {
                    balance: 42,
                    token_id: 8888_u256,
                };

                let mut payload: Array<felt252> = ArrayTrait::new();
                l1_data.serialize(ref payload);

                let mut l1_handler = L1HandlerTrait::new(
                    contract_address,
                    selector!("process_l1_message")
                );

                l1_handler.execute(0x123, payload.span()).unwrap();
                    
                let dispatcher = IBalanceTokenDispatcher { contract_address };
                assert(dispatcher.get_balance() == 42, dispatcher.get_balance());
                assert(dispatcher.get_token_id() == 8888_u256, 'Invalid token id');
            }
             
            #[test]
            fn l1_handler_execute_panicking() {
                let calldata = array![0x123];

                let contract = declare("l1_handler_executor").unwrap();
                let (contract_address, _) = contract.deploy(@calldata).unwrap();


                let mut l1_handler = L1HandlerTrait::new(
                    contract_address,
                    selector!("panicking_l1_handler")
                );

                match l1_handler.execute(0x123, array![].span()) {
                    Result::Ok(_) => panic_with_felt252('should have panicked'),
                    Result::Err(panic_data) => {
                        assert(*panic_data.at(0) == 'custom', 'Wrong 1st panic datum');
                        assert(*panic_data.at(1) == 'panic', 'Wrong 2nd panic datum');
                    },
                }
            }

            #[test]
            fn l1_handler_function_missing() {
                let calldata = array![0x123];

                let contract = declare("l1_handler_executor").unwrap();
                let (contract_address, _) = contract.deploy(@calldata).unwrap();


                let mut l1_handler = L1HandlerTrait::new(
                    contract_address,
                    selector!("this_does_not_exist")
                );

                match l1_handler.execute(0x123, array![].span()){
                    Result::Ok(_) => panic_with_felt252('should have panicked'),
                    Result::Err(_) => {
                        // Would be nice to assert the error here once it is be possible in cairo
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

    assert_passed(&result);
}
