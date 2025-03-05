use forge_runner::forge_config::ForgeTrackedResource;
use indoc::indoc;
use std::path::Path;
use test_utils::runner::{Contract, assert_passed};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
fn cheat_block_number_basic() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{
                declare, ContractClassTrait, DeclareResultTrait, start_cheat_block_number,
                stop_cheat_block_number, stop_cheat_block_number_global, start_cheat_block_number_global
            };

            #[starknet::interface]
            trait ICheatBlockNumberChecker<TContractState> {
                fn get_block_number(ref self: TContractState) -> u64;
            }

            #[test]
            fn test_stop_cheat_block_number() {
                let contract = declare("CheatBlockNumberChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ICheatBlockNumberCheckerDispatcher { contract_address };

                let old_block_number = dispatcher.get_block_number();

                start_cheat_block_number(contract_address, 234);

                let new_block_number = dispatcher.get_block_number();
                assert(new_block_number == 234, 'Wrong block number');

                stop_cheat_block_number(contract_address);

                let new_block_number = dispatcher.get_block_number();
                assert(new_block_number == old_block_number, 'Block num did not change back');
            }

            #[test]
            fn test_cheat_block_number_multiple() {
                let contract = declare("CheatBlockNumberChecker").unwrap().contract_class();

                let (contract_address1, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let (contract_address2, _) = contract.deploy(@ArrayTrait::new()).unwrap();

                let cheat_block_number_checker1 = ICheatBlockNumberCheckerDispatcher { contract_address: contract_address1 };
                let cheat_block_number_checker2 = ICheatBlockNumberCheckerDispatcher { contract_address: contract_address2 };

                let old_block_number2 = cheat_block_number_checker2.get_block_number();

                start_cheat_block_number(cheat_block_number_checker1.contract_address, 123);

                let new_block_number1 = cheat_block_number_checker1.get_block_number();
                let new_block_number2 = cheat_block_number_checker2.get_block_number();

                assert(new_block_number1 == 123, 'Wrong block number #1');
                assert(new_block_number2 == old_block_number2, 'Wrong block number #2');

                stop_cheat_block_number(cheat_block_number_checker2.contract_address);

                let new_block_number1 = cheat_block_number_checker1.get_block_number();
                let new_block_number2 = cheat_block_number_checker2.get_block_number();

                assert(new_block_number1 == 123, 'Wrong block number #1');
                assert(new_block_number2 == old_block_number2, 'Wrong block number #2');
            }

            #[test]
            fn test_cheat_block_number_all() {
                let contract = declare("CheatBlockNumberChecker").unwrap().contract_class();

                let (contract_address1, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let (contract_address2, _) = contract.deploy(@ArrayTrait::new()).unwrap();

                let cheat_block_number_checker1 = ICheatBlockNumberCheckerDispatcher { contract_address: contract_address1 };
                let cheat_block_number_checker2 = ICheatBlockNumberCheckerDispatcher { contract_address: contract_address2 };

                let old_block_number1 = cheat_block_number_checker1.get_block_number();
                let old_block_number2 = cheat_block_number_checker2.get_block_number();

                start_cheat_block_number_global(123);

                let new_block_number1 = cheat_block_number_checker1.get_block_number();
                let new_block_number2 = cheat_block_number_checker2.get_block_number();

                assert(new_block_number1 == 123, 'Wrong block number #1');
                assert(new_block_number2 == 123, 'Wrong block number #2');

                stop_cheat_block_number_global();

                let new_block_number1 = cheat_block_number_checker1.get_block_number();
                let new_block_number2 = cheat_block_number_checker2.get_block_number();

                assert(new_block_number1 == old_block_number1, 'CheatBlockNumber not stopped #1');
                assert(new_block_number2 == old_block_number2, 'CheatBlockNumber not stopped #2');
            }

            #[test]
            fn test_cheat_block_number_all_stop_one() {
                let contract = declare("CheatBlockNumberChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ICheatBlockNumberCheckerDispatcher { contract_address };

                let old_block_number = dispatcher.get_block_number();

                start_cheat_block_number_global(234);

                let new_block_number = dispatcher.get_block_number();
                assert(new_block_number == 234, 'Wrong block number');

                stop_cheat_block_number(contract_address);

                let new_block_number = dispatcher.get_block_number();
                assert(new_block_number == old_block_number, 'Block num did not change back');
            }
        "#
        ),
        Contract::from_code_path(
            "CheatBlockNumberChecker".to_string(),
            Path::new("tests/data/contracts/cheat_block_number_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn cheat_block_number_complex() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait, start_cheat_block_number, stop_cheat_block_number, start_cheat_block_number_global, stop_cheat_block_number_global };

            #[starknet::interface]
            trait ICheatBlockNumberChecker<TContractState> {
                fn get_block_number(ref self: TContractState) -> u64;
            }

            fn deploy_cheat_block_number_checker()  -> ICheatBlockNumberCheckerDispatcher {
                let contract = declare("CheatBlockNumberChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                ICheatBlockNumberCheckerDispatcher { contract_address }
            }

            #[test]
            fn cheat_block_number_complex() {
                let contract = declare("CheatBlockNumberChecker").unwrap().contract_class();

                let (contract_address1, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let (contract_address2, _) = contract.deploy(@ArrayTrait::new()).unwrap();

                let cheat_block_number_checker1 = ICheatBlockNumberCheckerDispatcher { contract_address: contract_address1 };
                let cheat_block_number_checker2 = ICheatBlockNumberCheckerDispatcher { contract_address: contract_address2 };

                let old_block_number2 = cheat_block_number_checker2.get_block_number();

                start_cheat_block_number_global(123);

                let new_block_number1 = cheat_block_number_checker1.get_block_number();
                let new_block_number2 = cheat_block_number_checker2.get_block_number();

                assert(new_block_number1 == 123, 'Wrong block number #1');
                assert(new_block_number2 == 123, 'Wrong block number #2');

                start_cheat_block_number(cheat_block_number_checker1.contract_address, 456);

                let new_block_number1 = cheat_block_number_checker1.get_block_number();
                let new_block_number2 = cheat_block_number_checker2.get_block_number();

                assert(new_block_number1 == 456, 'Wrong block number #3');
                assert(new_block_number2 == 123, 'Wrong block number #4');

                start_cheat_block_number(cheat_block_number_checker1.contract_address, 789);
                start_cheat_block_number(cheat_block_number_checker2.contract_address, 789);

                let new_block_number1 = cheat_block_number_checker1.get_block_number();
                let new_block_number2 = cheat_block_number_checker2.get_block_number();

                assert(new_block_number1 == 789, 'Wrong block number #5');
                assert(new_block_number2 == 789, 'Wrong block number #6');

                stop_cheat_block_number(cheat_block_number_checker2.contract_address);

                let new_block_number1 = cheat_block_number_checker1.get_block_number();
                let new_block_number2 = cheat_block_number_checker2.get_block_number();

                assert(new_block_number1 == 789, 'Wrong block number #7');
                assert(new_block_number2 == old_block_number2, 'Wrong block number #8');
            }
        "#
        ),
        Contract::from_code_path(
            "CheatBlockNumberChecker".to_string(),
            Path::new("tests/data/contracts/cheat_block_number_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn cheat_block_number_with_span() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ test_address, declare, ContractClassTrait, DeclareResultTrait, cheat_block_number, start_cheat_block_number, stop_cheat_block_number, CheatSpan };

            #[starknet::interface]
            trait ICheatBlockNumberChecker<TContractState> {
                fn get_block_number(ref self: TContractState) -> felt252;
            }

            fn deploy_cheat_block_number_checker() -> ICheatBlockNumberCheckerDispatcher {
                let (contract_address, _) = declare("CheatBlockNumberChecker").unwrap().contract_class().deploy(@ArrayTrait::new()).unwrap();
                ICheatBlockNumberCheckerDispatcher { contract_address }
            }

            #[test]
            fn test_cheat_block_number_once() {
                let old_block_number = get_block_number();

                let dispatcher = deploy_cheat_block_number_checker();

                let target_block_number = 123;

                cheat_block_number(dispatcher.contract_address, target_block_number, CheatSpan::TargetCalls(1));

                let block_number = dispatcher.get_block_number();
                assert_eq!(block_number, target_block_number.into());

                let block_number = dispatcher.get_block_number();
                assert_eq!(block_number, old_block_number.into());
            }

            #[test]
            fn test_cheat_block_number_twice() {
                let old_block_number = get_block_number();

                let dispatcher = deploy_cheat_block_number_checker();

                let target_block_number = 123;

                cheat_block_number(dispatcher.contract_address, target_block_number, CheatSpan::TargetCalls(2));

                let block_number = dispatcher.get_block_number();
                assert_eq!(block_number, target_block_number.into());

                let block_number = dispatcher.get_block_number();
                assert_eq!(block_number, target_block_number.into());

                let block_number = dispatcher.get_block_number();
                assert_eq!(block_number, old_block_number.into());
            }

            #[test]
            fn test_cheat_block_number_test_address() {
                let old_block_number = get_block_number();

                let target_block_number = 123;

                cheat_block_number(test_address(), target_block_number, CheatSpan::TargetCalls(1));

                let block_number = get_block_number();
                assert_eq!(block_number, target_block_number.into());

                let block_number = get_block_number();
                assert_eq!(block_number, target_block_number.into());

                stop_cheat_block_number(test_address());

                let block_number = get_block_number();
                assert_eq!(block_number, old_block_number.into());
            }

            fn get_block_number() -> u64 {
                starknet::get_block_info().unbox().block_number
            }
        "#
        ),
        Contract::from_code_path(
            "CheatBlockNumberChecker".to_string(),
            Path::new("tests/data/contracts/cheat_block_number_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}
