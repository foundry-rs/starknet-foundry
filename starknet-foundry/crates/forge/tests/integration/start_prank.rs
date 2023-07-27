use crate::integration::common::corelib::{corelib, predeployed_contracts};
use crate::integration::common::runner::Contract;
use crate::{assert_passed, test_case};
use camino::Utf8PathBuf;
use forge::run;
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
            use cheatcodes::PreparedContract;

            #[starknet::interface]
            trait IPrankChecker<TContractState> {
                fn get_caller_address(ref self: TContractState) -> felt252;
            }

            #[test]
            fn test_prank_simple() {
                let class_hash = declare('PrankChecker').unwrap();
                let prepared = PreparedContract { class_hash, constructor_calldata: @ArrayTrait::new() };
                let contract_address = deploy(prepared).unwrap();
                let contract_address: ContractAddress = contract_address.try_into().unwrap();
                let dispatcher = IPrankCheckerDispatcher { contract_address };
            
                let caller_address: felt252 = 123;
                let caller_address: ContractAddress = caller_address.try_into().unwrap();

                start_prank(caller_address, contract_address);
            
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

    let result = run(
        &test.path().unwrap(),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        Some(&Utf8PathBuf::from_path_buf(corelib().to_path_buf()).unwrap()),
        &test.contracts(corelib().path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();

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
            use cheatcodes::PreparedContract;
            
            #[starknet::interface]
            trait IPrankChecker<TContractState> {
                fn get_caller_address_and_emit_event(ref self: TContractState) -> felt252;
            }

            #[test]
            fn test_prank_with_other_syscall() {
                let class_hash = declare('PrankChecker').unwrap();
                let prepared = PreparedContract { class_hash, constructor_calldata: @ArrayTrait::new() };
                let contract_address = deploy(prepared).unwrap();
                let contract_address: ContractAddress = contract_address.try_into().unwrap();
                let dispatcher = IPrankCheckerDispatcher { contract_address };
            
                let caller_address: felt252 = 123;
                let caller_address: ContractAddress = caller_address.try_into().unwrap();

                start_prank(caller_address, contract_address);

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

    let result = run(
        &test.path().unwrap(),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        Some(&Utf8PathBuf::from_path_buf(corelib().to_path_buf()).unwrap()),
        &test.contracts(corelib().path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();

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
            use cheatcodes::PreparedContract;
            use forge_print::PrintTrait;
            
            #[starknet::interface]
            trait IConstructorPrankChecker<TContractState> {
                fn get_stored_caller_address(ref self: TContractState) -> ContractAddress;
            }
            
            #[test]
            fn test_prank_constructor_simple() {
                let class_hash = declare('ConstructorPrankChecker').unwrap();
                let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @ArrayTrait::new() };

                // TODO (#254): Change to the actual address
                let contract_address: ContractAddress = 2598896470772924212281968896271340780432065735045468431712403008297614014532.try_into().unwrap();
                start_prank(555, contract_address);
                let contract_address: ContractAddress = deploy(prepared).unwrap().try_into().unwrap();
            
                let dispatcher = IConstructorPrankCheckerDispatcher { contract_address };
                assert(dispatcher.get_stored_block_number() == 555, 'Wrong stored caller address');
            }
        "#
        ),
        Contract::from_code_path(
            "ConstructorRollChecker".to_string(),
            Path::new("tests/data/contracts/constructor_prank_checker.cairo"),
        )
        .unwrap()
    );

    let result = run(
        &test.path().unwrap(),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        Some(&Utf8PathBuf::from_path_buf(corelib().to_path_buf()).unwrap()),
        &test.contracts(corelib().path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();

    assert_passed!(result);
}
