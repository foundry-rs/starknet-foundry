use indoc::indoc;
use std::path::Path;
use test_utils::runner::{assert_passed, Contract};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
#[ignore] //TODO global variant
fn roll_basic() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ declare, ContractAddress, ContractClassTrait, start_roll, stop_roll };

            #[starknet::interface]
            trait IRollChecker<TContractState> {
                fn get_block_number(ref self: TContractState) -> u64;
            }

            #[test]
            fn test_stop_roll() {
                let contract = declare("RollChecker").unwrap();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IRollCheckerDispatcher { contract_address };

                let old_block_number = dispatcher.get_block_number();

                start_roll(ContractAddress::One(contract_address), 234);

                let new_block_number = dispatcher.get_block_number();
                assert(new_block_number == 234, 'Wrong block number');

                stop_roll(ContractAddress::One(contract_address));

                let new_block_number = dispatcher.get_block_number();
                assert(new_block_number == old_block_number, 'Block num did not change back');
            }

            #[test]
            fn test_roll_multiple() {
                let contract = declare("RollChecker").unwrap();

                let (contract_address1, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let (contract_address2, _) = contract.deploy(@ArrayTrait::new()).unwrap();

                let roll_checker1 = IRollCheckerDispatcher { contract_address: contract_address1 };
                let roll_checker2 = IRollCheckerDispatcher { contract_address: contract_address2 };

                let old_block_number1 = roll_checker1.get_block_number();
                let old_block_number2 = roll_checker2.get_block_number();

                start_roll(ContractAddress::Multiple(array![roll_checker1.contract_address, roll_checker2.contract_address]), 123);

                let new_block_number1 = roll_checker1.get_block_number();
                let new_block_number2 = roll_checker2.get_block_number();

                assert(new_block_number1 == 123, 'Wrong block number #1');
                assert(new_block_number2 == 123, 'Wrong block number #2');

                stop_roll(ContractAddress::Multiple(array![roll_checker1.contract_address, roll_checker2.contract_address]));

                let new_block_number1 = roll_checker1.get_block_number();
                let new_block_number2 = roll_checker2.get_block_number();

                assert(new_block_number1 == old_block_number1, 'Roll not stopped #1');
                assert(new_block_number2 == old_block_number2, 'Roll not stopped #2');
            }

            #[test]
            fn test_roll_all() {
                let contract = declare("RollChecker").unwrap();

                let (contract_address1, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let (contract_address2, _) = contract.deploy(@ArrayTrait::new()).unwrap();

                let roll_checker1 = IRollCheckerDispatcher { contract_address: contract_address1 };
                let roll_checker2 = IRollCheckerDispatcher { contract_address: contract_address2 };

                let old_block_number1 = roll_checker1.get_block_number();
                let old_block_number2 = roll_checker2.get_block_number();

                start_roll(ContractAddress::All, 123);

                let new_block_number1 = roll_checker1.get_block_number();
                let new_block_number2 = roll_checker2.get_block_number();

                assert(new_block_number1 == 123, 'Wrong block number #1');
                assert(new_block_number2 == 123, 'Wrong block number #2');

                stop_roll(ContractAddress::All);

                let new_block_number1 = roll_checker1.get_block_number();
                let new_block_number2 = roll_checker2.get_block_number();

                assert(new_block_number1 == old_block_number1, 'Roll not stopped #1');
                assert(new_block_number2 == old_block_number2, 'Roll not stopped #2');
            }

            #[test]
            fn test_roll_all_stop_one() {
                let contract = declare("RollChecker").unwrap();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IRollCheckerDispatcher { contract_address };

                let old_block_number = dispatcher.get_block_number();

                start_roll(ContractAddress::All, 234);

                let new_block_number = dispatcher.get_block_number();
                assert(new_block_number == 234, 'Wrong block number');

                stop_roll(ContractAddress::One(contract_address));

                let new_block_number = dispatcher.get_block_number();
                assert(new_block_number == old_block_number, 'Block num did not change back');
            }
        "#
        ),
        Contract::from_code_path(
            "RollChecker".to_string(),
            Path::new("tests/data/contracts/roll_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
#[ignore] //TODO global variant
fn roll_complex() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ declare, ContractAddress, ContractClassTrait, start_roll, stop_roll };
            
            #[starknet::interface]
            trait IRollChecker<TContractState> {
                fn get_block_number(ref self: TContractState) -> u64;
            }

            fn deploy_roll_checker()  -> IRollCheckerDispatcher {
                let contract = declare("RollChecker").unwrap();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                IRollCheckerDispatcher { contract_address }
            }

            #[test]
            fn roll_complex() {
                let contract = declare("RollChecker").unwrap();

                let (contract_address1, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let (contract_address2, _) = contract.deploy(@ArrayTrait::new()).unwrap();

                let roll_checker1 = IRollCheckerDispatcher { contract_address: contract_address1 };
                let roll_checker2 = IRollCheckerDispatcher { contract_address: contract_address2 };

                let old_block_number2 = roll_checker2.get_block_number();

                start_roll(ContractAddress::All, 123);

                let new_block_number1 = roll_checker1.get_block_number();
                let new_block_number2 = roll_checker2.get_block_number();
                
                assert(new_block_number1 == 123, 'Wrong block number #1');
                assert(new_block_number2 == 123, 'Wrong block number #2');

                start_roll(ContractAddress::One(roll_checker1.contract_address), 456);

                let new_block_number1 = roll_checker1.get_block_number();
                let new_block_number2 = roll_checker2.get_block_number();

                assert(new_block_number1 == 456, 'Wrong block number #3');
                assert(new_block_number2 == 123, 'Wrong block number #4');

                start_roll(ContractAddress::Multiple(array![roll_checker1.contract_address, roll_checker2.contract_address]), 789);

                let new_block_number1 = roll_checker1.get_block_number();
                let new_block_number2 = roll_checker2.get_block_number();

                assert(new_block_number1 == 789, 'Wrong block number #5');
                assert(new_block_number2 == 789, 'Wrong block number #6');

                stop_roll(ContractAddress::One(roll_checker2.contract_address));

                let new_block_number1 = roll_checker1.get_block_number();
                let new_block_number2 = roll_checker2.get_block_number();
                
                assert(new_block_number1 == 789, 'Wrong block number #7');
                assert(new_block_number2 == old_block_number2, 'Wrong block number #8');
            }
        "#
        ),
        Contract::from_code_path(
            "RollChecker".to_string(),
            Path::new("tests/data/contracts/roll_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn roll_with_span() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ test_address, declare, ContractClassTrait, roll, start_roll, stop_roll, CheatSpan };

            #[starknet::interface]
            trait IRollChecker<TContractState> {
                fn get_block_number(ref self: TContractState) -> felt252;
            }

            fn deploy_roll_checker() -> IRollCheckerDispatcher {
                let (contract_address, _) = declare("RollChecker").unwrap().deploy(@ArrayTrait::new()).unwrap();
                IRollCheckerDispatcher { contract_address }
            }

            #[test]
            fn test_roll_once() {
                let old_block_number = get_block_number();
                
                let dispatcher = deploy_roll_checker();

                let target_block_number = 123;

                roll(dispatcher.contract_address, target_block_number, CheatSpan::TargetCalls(1));

                let block_number = dispatcher.get_block_number();
                assert_eq!(block_number, target_block_number.into());

                let block_number = dispatcher.get_block_number();
                assert_eq!(block_number, old_block_number.into());
            }

            #[test]
            fn test_roll_twice() {
                let old_block_number = get_block_number();

                let dispatcher = deploy_roll_checker();

                let target_block_number = 123;

                roll(dispatcher.contract_address, target_block_number, CheatSpan::TargetCalls(2));

                let block_number = dispatcher.get_block_number();
                assert_eq!(block_number, target_block_number.into());
                
                let block_number = dispatcher.get_block_number();
                assert_eq!(block_number, target_block_number.into());

                let block_number = dispatcher.get_block_number();
                assert_eq!(block_number, old_block_number.into());
            }

            #[test]
            fn test_roll_test_address() {
                let old_block_number = get_block_number();
                
                let target_block_number = 123;
                
                roll(test_address(), target_block_number, CheatSpan::TargetCalls(1));
                
                let block_number = get_block_number();
                assert_eq!(block_number, target_block_number.into());

                let block_number = get_block_number();
                assert_eq!(block_number, target_block_number.into());
                
                stop_roll(test_address());

                let block_number = get_block_number();
                assert_eq!(block_number, old_block_number.into());
            }

            fn get_block_number() -> u64 {
                starknet::get_block_info().unbox().block_number
            }
        "#
        ),
        Contract::from_code_path(
            "RollChecker".to_string(),
            Path::new("tests/data/contracts/roll_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}
