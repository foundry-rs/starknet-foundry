use indoc::indoc;
use std::path::Path;
use test_utils::runner::Contract;
use test_utils::running_tests::run_test_case;
use test_utils::{assert_passed, test_case};

#[test]
fn roll_basic() {
    let test = test_case!(
        indoc!(
            r"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ declare, CheatTarget, ContractClassTrait, start_roll, stop_roll };

            #[starknet::interface]
            trait IRollChecker<TContractState> {
                fn get_block_number(ref self: TContractState) -> u64;
            }

            #[test]
            fn test_stop_roll() {
                let contract = declare('RollChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IRollCheckerDispatcher { contract_address };

                let old_block_number = dispatcher.get_block_number();

                start_roll(CheatTarget::One(contract_address), 234);

                let new_block_number = dispatcher.get_block_number();
                assert(new_block_number == 234, 'Wrong block number');

                stop_roll(CheatTarget::One(contract_address));

                let new_block_number = dispatcher.get_block_number();
                assert(new_block_number == old_block_number, 'Block num did not change back');
            }

            #[test]
            fn test_roll_multiple() {
                let contract = declare('RollChecker');

                let roll_checker1 = IRollCheckerDispatcher { contract_address: contract.deploy(@ArrayTrait::new()).unwrap() };
                let roll_checker2 = IRollCheckerDispatcher { contract_address: contract.deploy(@ArrayTrait::new()).unwrap() };

                let old_block_number1 = roll_checker1.get_block_number();
                let old_block_number2 = roll_checker2.get_block_number();

                start_roll(CheatTarget::Multiple(array![roll_checker1.contract_address, roll_checker2.contract_address]), 123);

                let new_block_number1 = roll_checker1.get_block_number();
                let new_block_number2 = roll_checker2.get_block_number();

                assert(new_block_number1 == 123, 'Wrong block number #1');
                assert(new_block_number2 == 123, 'Wrong block number #2');

                stop_roll(CheatTarget::Multiple(array![roll_checker1.contract_address, roll_checker2.contract_address]));

                let new_block_number1 = roll_checker1.get_block_number();
                let new_block_number2 = roll_checker2.get_block_number();

                assert(new_block_number1 == old_block_number1, 'Roll not stopped #1');
                assert(new_block_number2 == old_block_number2, 'Roll not stopped #2');
            }

            #[test]
            fn test_roll_all() {
                let contract = declare('RollChecker');

                let roll_checker1 = IRollCheckerDispatcher { contract_address: contract.deploy(@ArrayTrait::new()).unwrap() };
                let roll_checker2 = IRollCheckerDispatcher { contract_address: contract.deploy(@ArrayTrait::new()).unwrap() };

                let old_block_number1 = roll_checker1.get_block_number();
                let old_block_number2 = roll_checker2.get_block_number();

                start_roll(CheatTarget::All, 123);

                let new_block_number1 = roll_checker1.get_block_number();
                let new_block_number2 = roll_checker2.get_block_number();

                assert(new_block_number1 == 123, 'Wrong block number #1');
                assert(new_block_number2 == 123, 'Wrong block number #2');

                stop_roll(CheatTarget::All);

                let new_block_number1 = roll_checker1.get_block_number();
                let new_block_number2 = roll_checker2.get_block_number();

                assert(new_block_number1 == old_block_number1, 'Roll not stopped #1');
                assert(new_block_number2 == old_block_number2, 'Roll not stopped #2');
            }

            #[test]
            fn test_roll_all_stop_one() {
                let contract = declare('RollChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IRollCheckerDispatcher { contract_address };

                let old_block_number = dispatcher.get_block_number();

                start_roll(CheatTarget::All, 234);

                let new_block_number = dispatcher.get_block_number();
                assert(new_block_number == 234, 'Wrong block number');

                stop_roll(CheatTarget::One(contract_address));

                let new_block_number = dispatcher.get_block_number();
                assert(new_block_number == old_block_number, 'Block num did not change back');
            }
        "
        ),
        Contract::from_code_path(
            "RollChecker".to_string(),
            Path::new("tests/data/contracts/roll_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn roll_complex() {
    let test = test_case!(
        indoc!(
            r"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ declare, CheatTarget, ContractClassTrait, start_roll, stop_roll };
            
            #[starknet::interface]
            trait IRollChecker<TContractState> {
                fn get_block_number(ref self: TContractState) -> u64;
            }

            fn deploy_roll_checker()  -> IRollCheckerDispatcher {
                let contract = declare('RollChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                IRollCheckerDispatcher { contract_address }
            }

            #[test]
            fn roll_complex() {
                let contract = declare('RollChecker');

                let roll_checker1 = IRollCheckerDispatcher { contract_address: contract.deploy(@ArrayTrait::new()).unwrap() };
                let roll_checker2 = IRollCheckerDispatcher { contract_address: contract.deploy(@ArrayTrait::new()).unwrap() };

                let old_block_number2 = roll_checker2.get_block_number();

                start_roll(CheatTarget::All, 123);

                let new_block_number1 = roll_checker1.get_block_number();
                let new_block_number2 = roll_checker2.get_block_number();
                
                assert(new_block_number1 == 123, 'Wrong block number #1');
                assert(new_block_number2 == 123, 'Wrong block number #2');

                start_roll(CheatTarget::One(roll_checker1.contract_address), 456);

                let new_block_number1 = roll_checker1.get_block_number();
                let new_block_number2 = roll_checker2.get_block_number();

                assert(new_block_number1 == 456, 'Wrong block number #3');
                assert(new_block_number2 == 123, 'Wrong block number #4');

                start_roll(CheatTarget::Multiple(array![roll_checker1.contract_address, roll_checker2.contract_address]), 789);

                let new_block_number1 = roll_checker1.get_block_number();
                let new_block_number2 = roll_checker2.get_block_number();

                assert(new_block_number1 == 789, 'Wrong block number #5');
                assert(new_block_number2 == 789, 'Wrong block number #6');

                stop_roll(CheatTarget::One(roll_checker2.contract_address));

                let new_block_number1 = roll_checker1.get_block_number();
                let new_block_number2 = roll_checker2.get_block_number();
                
                assert(new_block_number1 == 789, 'Wrong block number #7');
                assert(new_block_number2 == old_block_number2, 'Wrong block number #8');
            }
        "
        ),
        Contract::from_code_path(
            "RollChecker".to_string(),
            Path::new("tests/data/contracts/roll_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}
