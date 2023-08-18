use crate::integration::common::runner::Contract;
use crate::integration::common::running_tests::run_test_case;
use crate::{assert_passed, test_case};
use indoc::indoc;
use std::path::Path;

#[test]
fn start_prank_simple() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ declare, ContractClassTrait, start_prank };

            #[starknet::interface]
            trait IPrankChecker<TContractState> {
                fn get_caller_address(ref self: TContractState) -> felt252;
            }

            #[test]
            fn test_prank_simple() {
                let contract = declare('PrankChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IPrankCheckerDispatcher { contract_address };

                let caller_address: felt252 = 123;
                let caller_address: ContractAddress = caller_address.try_into().unwrap();

                start_prank(contract_address, caller_address);

                let caller_address = dispatcher.get_caller_address();
                assert(caller_address == 123, 'Wrong caller address');
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

    assert_passed!(result);
}

#[test]
fn start_prank_with_other_syscall() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ declare, ContractClassTrait, start_prank };

            #[starknet::interface]
            trait IPrankChecker<TContractState> {
                fn get_caller_address_and_emit_event(ref self: TContractState) -> felt252;
            }

            #[test]
            fn test_prank_with_other_syscall() {
                let contract = declare('PrankChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IPrankCheckerDispatcher { contract_address };

                let caller_address: felt252 = 123;
                let caller_address: ContractAddress = caller_address.try_into().unwrap();

                start_prank(contract_address, caller_address);

                let caller_address = dispatcher.get_caller_address_and_emit_event();
                assert(caller_address == 123, 'Wrong caller address');
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

    assert_passed!(result);
}

// TODO (#254): Make it pass
#[test]
#[ignore]
fn start_prank_in_constructor_test() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ declare, ContractClassTrait, start_prank };

            #[starknet::interface]
            trait IConstructorPrankChecker<TContractState> {
                fn get_stored_caller_address(ref self: TContractState) -> ContractAddress;
            }

            #[test]
            fn test_prank_constructor_simple() {
                let contract = declare('ConstructorPrankChecker');

                // TODO (#254): Change to the actual address
                let contract_address: ContractAddress = 2598896470772924212281968896271340780432065735045468431712403008297614014532.try_into().unwrap();
                start_prank(contract_address, 555);
                let contract_address: ContractAddress = contract.deploy(@ArrayTrait::new()).unwrap().try_into().unwrap();

                let dispatcher = IConstructorPrankCheckerDispatcher { contract_address };
                assert(dispatcher.get_stored_block_number() == 555, 'Wrong stored caller address');
            }
        "#
        ),
        Contract::from_code_path(
            "ConstructorPrankChecker".to_string(),
            Path::new("tests/data/contracts/constructor_prank_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn stop_prank() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ declare, ContractClassTrait, start_prank, stop_prank };

            #[starknet::interface]
            trait IPrankChecker<TContractState> {
                fn get_caller_address(ref self: TContractState) -> felt252;
            }

            #[test]
            fn test_stop_prank() {
                let contract = declare('PrankChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IPrankCheckerDispatcher { contract_address };

                let target_caller_address: felt252 = 123;
                let target_caller_address: ContractAddress = target_caller_address.try_into().unwrap();

                let old_caller_address = dispatcher.get_caller_address();

                start_prank(contract_address, target_caller_address);

                let new_caller_address = dispatcher.get_caller_address();
                assert(new_caller_address == 123, 'Wrong caller address');

                stop_prank(contract_address);

                let new_caller_address = dispatcher.get_caller_address();
                assert(old_caller_address == new_caller_address, 'Address did not change back');
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

    assert_passed!(result);
}

#[test]
fn double_prank() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ declare, ContractClassTrait, start_prank, stop_prank };


            #[starknet::interface]
            trait IPrankChecker<TContractState> {
                fn get_caller_address(ref self: TContractState) -> felt252;
            }

            #[test]
            fn test_stop_prank() {
                let contract = declare('PrankChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IPrankCheckerDispatcher { contract_address };

                let target_caller_address: felt252 = 123;
                let target_caller_address: ContractAddress = target_caller_address.try_into().unwrap();

                let old_caller_address = dispatcher.get_caller_address();

                start_prank(contract_address, target_caller_address);
                start_prank(contract_address, target_caller_address);

                let new_caller_address = dispatcher.get_caller_address();
                assert(new_caller_address == 123, 'Wrong caller address');

                stop_prank(contract_address);

                let new_caller_address = dispatcher.get_caller_address();
                assert(old_caller_address == new_caller_address, 'Address did not change back');
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

    assert_passed!(result);
}

#[test]
fn start_prank_with_proxy() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use traits::Into;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ declare, ContractClassTrait, start_prank, stop_prank };

            #[starknet::interface]
            trait IPrankCheckerProxy<TContractState> {
                fn get_prank_checkers_caller_address(ref self: TContractState, address: ContractAddress) -> felt252;
            }
            #[test]
            fn test_prank_simple() {
                let contract = declare('PrankChecker');
                let prank_checker_contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let contract_address: ContractAddress = 234.try_into().unwrap();
                start_prank(prank_checker_contract_address, contract_address);

                let contract = declare('PrankCheckerProxy');
                let proxy_contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let proxy_dispatcher = IPrankCheckerProxyDispatcher { contract_address: proxy_contract_address };
                let caller_address = proxy_dispatcher.get_prank_checkers_caller_address(prank_checker_contract_address);
                assert(caller_address == 234, caller_address);
            }
        "#
        ),
        Contract::from_code_path(
            "PrankChecker".to_string(),
            Path::new("tests/data/contracts/prank_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "PrankCheckerProxy".to_string(),
            Path::new("tests/data/contracts/prank_checker_proxy.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn start_prank_with_library_call() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use traits::Into;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ declare, ContractClassTrait, start_prank };

            use starknet::ClassHash;

            #[starknet::interface]
            trait IPrankCheckerLibCall<TContractState> {
                fn get_caller_address_with_lib_call(ref self: TContractState, class_hash: ClassHash) -> felt252;
            }

            #[test]
            fn test_prank_simple() {
                let prank_checker_contract = declare('PrankChecker');
                let prank_checker_class_hash = prank_checker_contract.class_hash.into();

                let contract = declare('PrankCheckerLibCall');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();

                let pranked_address: ContractAddress = 234.try_into().unwrap();
                start_prank(contract_address, pranked_address);

                let dispatcher = IPrankCheckerLibCallDispatcher { contract_address };
                let caller_address = dispatcher.get_caller_address_with_lib_call(prank_checker_class_hash);
                assert(caller_address == 234, caller_address);
            }
        "#
        ),
        Contract::from_code_path(
            "PrankChecker".to_string(),
            Path::new("tests/data/contracts/prank_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "PrankCheckerLibCall".to_string(),
            Path::new("tests/data/contracts/prank_checker_library_call.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}
