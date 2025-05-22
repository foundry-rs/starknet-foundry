use forge_runner::forge_config::ForgeTrackedResource;
use foundry_ui::Ui;
use indoc::indoc;
use std::path::Path;
use test_utils::runner::{Contract, assert_passed};
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
            use snforge_std::{declare, ContractClassTrait, DeclareResultTrait, L1Handler, L1HandlerTrait};
            use starknet::contract_address_const;

            #[test]
            fn l1_handler_execute() {
                let calldata = array![0x123];

                let contract = declare("l1_handler_executor").unwrap().contract_class();
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

                let contract = declare("l1_handler_executor").unwrap().contract_class();
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

                let contract = declare("l1_handler_executor").unwrap().contract_class();
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

            #[test]
            #[should_panic]
            fn l1_handler_contract_missing() {
                let dispatcher = IBalanceTokenDispatcher { contract_address: contract_address_const::<421984739218742310>() };
                dispatcher.get_balance();

                let mut l1_handler = L1HandlerTrait::new(
                    contract_address_const::<421984739218742310>(),
                    selector!("process_l1_message")
                );

                l1_handler.execute(0x123, array![].span());
            }
        "#
        ),
        Contract::from_code_path(
            "l1_handler_executor".to_string(),
            Path::new("tests/data/contracts/l1_handler_execute_checker.cairo"),
        )
        .unwrap()
    );

    let ui = Ui::default();
    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps, &ui);

    assert_passed(&result);
}
