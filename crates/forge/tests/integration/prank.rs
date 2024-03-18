use indoc::indoc;
use std::path::Path;
use test_utils::runner::{assert_passed, Contract};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
fn prank() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ declare, ContractClassTrait, start_prank, stop_prank, CheatTarget,};

            #[starknet::interface]
            trait IPrankChecker<TContractState> {
                fn get_caller_address(ref self: TContractState) -> felt252;
            }

            #[test]
            fn test_stop_prank() {
                let contract = declare("PrankChecker");
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IPrankCheckerDispatcher { contract_address };

                let target_caller_address: felt252 = 123;
                let target_caller_address: ContractAddress = target_caller_address.try_into().unwrap();

                let old_caller_address = dispatcher.get_caller_address();

                start_prank(CheatTarget::One(contract_address), target_caller_address);

                let new_caller_address = dispatcher.get_caller_address();
                assert(new_caller_address == 123, 'Wrong caller address');

                stop_prank(CheatTarget::One(contract_address));

                let new_caller_address = dispatcher.get_caller_address();
                assert(old_caller_address == new_caller_address, 'Address did not change back');
            }

            #[test]
            fn test_prank_all() {
                let contract = declare("PrankChecker");
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IPrankCheckerDispatcher { contract_address };

                let target_caller_address: felt252 = 123;
                let target_caller_address: ContractAddress = target_caller_address.try_into().unwrap();

                let old_caller_address = dispatcher.get_caller_address();

                start_prank(CheatTarget::All, target_caller_address);

                let new_caller_address = dispatcher.get_caller_address();
                assert(new_caller_address == 123, 'Wrong caller address');

                stop_prank(CheatTarget::All);

                let new_caller_address = dispatcher.get_caller_address();
                assert(old_caller_address == new_caller_address, 'Address did not change back');
            }

            #[test]
            fn test_prank_all_stop_one() {
                let contract = declare("PrankChecker");
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IPrankCheckerDispatcher { contract_address };

                let target_caller_address: felt252 = 123;
                let target_caller_address: ContractAddress = target_caller_address.try_into().unwrap();

                let old_caller_address = dispatcher.get_caller_address();

                start_prank(CheatTarget::All, target_caller_address);

                let new_caller_address = dispatcher.get_caller_address();
                assert(new_caller_address == 123, 'Wrong caller address');

                stop_prank(CheatTarget::One(contract_address));

                let new_caller_address = dispatcher.get_caller_address();
                assert(old_caller_address == new_caller_address, 'Address did not change back');
            }

            #[test]
            fn test_prank_multiple() {
                let contract = declare("PrankChecker");

                let contract_address1 = contract.deploy(@ArrayTrait::new()).unwrap();
                let contract_address2 = contract.deploy(@ArrayTrait::new()).unwrap();

                let dispatcher1 = IPrankCheckerDispatcher { contract_address: contract_address1 };
                let dispatcher2 = IPrankCheckerDispatcher { contract_address: contract_address2 };

                let target_caller_address: felt252 = 123;
                let target_caller_address: ContractAddress = target_caller_address.try_into().unwrap();

                let old_caller_address1 = dispatcher1.get_caller_address();
                let old_caller_address2 = dispatcher2.get_caller_address();

                start_prank(CheatTarget::Multiple(array![contract_address1, contract_address2]), target_caller_address);

                let new_caller_address1 = dispatcher1.get_caller_address();
                let new_caller_address2 = dispatcher2.get_caller_address();

                assert(new_caller_address1 == 123, 'Wrong caller address #1');
                assert(new_caller_address2 == 123, 'Wrong caller address #2');

                stop_prank(CheatTarget::Multiple(array![contract_address1, contract_address2]));

                let new_caller_address1 = dispatcher1.get_caller_address();
                let new_caller_address2 = dispatcher2.get_caller_address();

                assert(old_caller_address1 == new_caller_address1, 'Address did not change back #1');
                assert(old_caller_address2 == new_caller_address2, 'Address did not change back #2');
            }
        "#
        ),
        Contract::from_code_path(
            "PrankChecker".to_string(),
            Path::new("tests/data/contracts/prank_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn prank_with_span() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ test_address, declare, ContractClassTrait, prank, start_prank, stop_prank, CheatTarget, CheatSpan };

            #[starknet::interface]
            trait IPrankChecker<TContractState> {
                fn get_caller_address(ref self: TContractState) -> felt252;
            }

            fn deploy_prank_checker() -> IPrankCheckerDispatcher {
                let contract_address = declare("PrankChecker").deploy(@ArrayTrait::new()).unwrap();
                IPrankCheckerDispatcher { contract_address }
            }

            #[test]
            fn test_prank_once() {
                let dispatcher = deploy_prank_checker();

                let target_caller_address: ContractAddress = 123.try_into().unwrap();

                prank(CheatTarget::One(dispatcher.contract_address), target_caller_address, CheatSpan::TargetCalls(1));

                let caller_address = dispatcher.get_caller_address();
                assert(caller_address == target_caller_address.into(), 'Wrong caller address');

                let caller_address = dispatcher.get_caller_address();
                assert(caller_address == test_address().into(), 'Address did not change back');
            }

            #[test]
            fn test_prank_twice() {
                let dispatcher = deploy_prank_checker();

                let target_caller_address: ContractAddress = 123.try_into().unwrap();

                prank(CheatTarget::One(dispatcher.contract_address), target_caller_address, CheatSpan::TargetCalls(2));

                let caller_address = dispatcher.get_caller_address();
                assert(caller_address == target_caller_address.into(), 'Wrong caller address');
                
                let caller_address = dispatcher.get_caller_address();
                assert(caller_address == target_caller_address.into(), 'Wrong caller address');

                let caller_address = dispatcher.get_caller_address();
                assert(caller_address == test_address().into(), 'Address did not change back');
            }

            #[test]
            fn test_prank_test_address() {
                let old_caller_address = starknet::get_caller_address();
                
                let target_caller_address: ContractAddress = 123.try_into().unwrap();
                
                prank(CheatTarget::One(test_address()), target_caller_address, CheatSpan::TargetCalls(1));
                
                let caller_address = starknet::get_caller_address();
                assert(caller_address == target_caller_address, 'Wrong caller address');

                let caller_address = starknet::get_caller_address();
                assert(caller_address == target_caller_address, 'Wrong caller address');
                
                stop_prank(CheatTarget::One(test_address()));

                let caller_address = starknet::get_caller_address();
                assert(caller_address == old_caller_address, 'Wrong caller address');
            }
        "#
        ),
        Contract::from_code_path(
            "PrankChecker".to_string(),
            Path::new("tests/data/contracts/prank_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}
