use forge_runner::forge_config::ForgeTrackedResource;
use indoc::indoc;
use std::path::Path;
use test_utils::runner::{Contract, assert_passed};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
fn cheat_caller_address() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait, start_cheat_caller_address, stop_cheat_caller_address, stop_cheat_caller_address_global, start_cheat_caller_address_global };

            #[starknet::interface]
            trait ICheatCallerAddressChecker<TContractState> {
                fn get_caller_address(ref self: TContractState) -> felt252;
            }

            #[test]
            fn test_stop_cheat_caller_address() {
                let contract = declare("CheatCallerAddressChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ICheatCallerAddressCheckerDispatcher { contract_address };

                let target_caller_address: felt252 = 123;
                let target_caller_address: ContractAddress = target_caller_address.try_into().unwrap();

                let old_caller_address = dispatcher.get_caller_address();

                start_cheat_caller_address(contract_address, target_caller_address);

                let new_caller_address = dispatcher.get_caller_address();
                assert(new_caller_address == 123, 'Wrong caller address');

                stop_cheat_caller_address(contract_address);

                let new_caller_address = dispatcher.get_caller_address();
                assert(old_caller_address == new_caller_address, 'Address did not change back');
            }

            #[test]
            fn test_cheat_caller_address_all() {
                let contract = declare("CheatCallerAddressChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ICheatCallerAddressCheckerDispatcher { contract_address };

                let target_caller_address: felt252 = 123;
                let target_caller_address: ContractAddress = target_caller_address.try_into().unwrap();

                let old_caller_address = dispatcher.get_caller_address();

                start_cheat_caller_address_global(target_caller_address);

                let new_caller_address = dispatcher.get_caller_address();
                assert(new_caller_address == 123, 'Wrong caller address');

                stop_cheat_caller_address_global();

                let new_caller_address = dispatcher.get_caller_address();
                assert(new_caller_address == old_caller_address, 'Wrong caller address');
            }

            #[test]
            fn test_cheat_caller_address_all_stop_one() {
                let contract = declare("CheatCallerAddressChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ICheatCallerAddressCheckerDispatcher { contract_address };

                let target_caller_address: felt252 = 123;
                let target_caller_address: ContractAddress = target_caller_address.try_into().unwrap();

                let old_caller_address = dispatcher.get_caller_address();

                start_cheat_caller_address_global(target_caller_address);

                let new_caller_address = dispatcher.get_caller_address();
                assert(new_caller_address == 123, 'Wrong caller address');

                stop_cheat_caller_address(contract_address);

                let new_caller_address = dispatcher.get_caller_address();
                assert(old_caller_address == new_caller_address, 'Address did not change back');
            }

            #[test]
            fn test_cheat_caller_address_multiple() {
                let contract = declare("CheatCallerAddressChecker").unwrap().contract_class();

                let (contract_address1, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let (contract_address2, _) = contract.deploy(@ArrayTrait::new()).unwrap();

                let dispatcher1 = ICheatCallerAddressCheckerDispatcher { contract_address: contract_address1 };
                let dispatcher2 = ICheatCallerAddressCheckerDispatcher { contract_address: contract_address2 };

                let target_caller_address: felt252 = 123;
                let target_caller_address: ContractAddress = target_caller_address.try_into().unwrap();

                let old_caller_address1 = dispatcher1.get_caller_address();
                let old_caller_address2 = dispatcher2.get_caller_address();

                start_cheat_caller_address(contract_address1, target_caller_address);
                start_cheat_caller_address(contract_address2, target_caller_address);

                let new_caller_address1 = dispatcher1.get_caller_address();
                let new_caller_address2 = dispatcher2.get_caller_address();

                assert(new_caller_address1 == 123, 'Wrong caller address #1');
                assert(new_caller_address2 == 123, 'Wrong caller address #2');

                stop_cheat_caller_address(contract_address1);
                stop_cheat_caller_address(contract_address2);

                let new_caller_address1 = dispatcher1.get_caller_address();
                let new_caller_address2 = dispatcher2.get_caller_address();

                assert(old_caller_address1 == new_caller_address1, 'Address did not change back #1');
                assert(old_caller_address2 == new_caller_address2, 'Address did not change back #2');
            }
        "#
        ),
        Contract::from_code_path(
            "CheatCallerAddressChecker".to_string(),
            Path::new("tests/data/contracts/cheat_caller_address_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn cheat_caller_address_with_span() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ test_address, declare, ContractClassTrait, DeclareResultTrait, cheat_caller_address, start_cheat_caller_address, stop_cheat_caller_address, CheatSpan };

            #[starknet::interface]
            trait ICheatCallerAddressChecker<TContractState> {
                fn get_caller_address(ref self: TContractState) -> felt252;
            }

            fn deploy_cheat_caller_address_checker() -> ICheatCallerAddressCheckerDispatcher {
                let (contract_address, _) = declare("CheatCallerAddressChecker").unwrap().contract_class().deploy(@ArrayTrait::new()).unwrap();
                ICheatCallerAddressCheckerDispatcher { contract_address }
            }

            #[test]
            fn test_cheat_caller_address_once() {
                let dispatcher = deploy_cheat_caller_address_checker();

                let target_caller_address: ContractAddress = 123.try_into().unwrap();

                cheat_caller_address(dispatcher.contract_address, target_caller_address, CheatSpan::TargetCalls(1));

                let caller_address = dispatcher.get_caller_address();
                assert(caller_address == target_caller_address.into(), 'Wrong caller address');

                let caller_address = dispatcher.get_caller_address();
                assert(caller_address == test_address().into(), 'Address did not change back');
            }

            #[test]
            fn test_cheat_caller_address_twice() {
                let dispatcher = deploy_cheat_caller_address_checker();

                let target_caller_address: ContractAddress = 123.try_into().unwrap();

                cheat_caller_address(dispatcher.contract_address, target_caller_address, CheatSpan::TargetCalls(2));

                let caller_address = dispatcher.get_caller_address();
                assert(caller_address == target_caller_address.into(), 'Wrong caller address');

                let caller_address = dispatcher.get_caller_address();
                assert(caller_address == target_caller_address.into(), 'Wrong caller address');

                let caller_address = dispatcher.get_caller_address();
                assert(caller_address == test_address().into(), 'Address did not change back');
            }

            #[test]
            fn test_cheat_caller_address_test_address() {
                let old_caller_address = starknet::get_caller_address();

                let target_caller_address: ContractAddress = 123.try_into().unwrap();

                cheat_caller_address(test_address(), target_caller_address, CheatSpan::TargetCalls(1));

                let caller_address = starknet::get_caller_address();
                assert(caller_address == target_caller_address, 'Wrong caller address');

                let caller_address = starknet::get_caller_address();
                assert(caller_address == target_caller_address, 'Wrong caller address');

                stop_cheat_caller_address(test_address());

                let caller_address = starknet::get_caller_address();
                assert(caller_address == old_caller_address, 'Wrong caller address');
            }
        "#
        ),
        Contract::from_code_path(
            "CheatCallerAddressChecker".to_string(),
            Path::new("tests/data/contracts/cheat_caller_address_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}
