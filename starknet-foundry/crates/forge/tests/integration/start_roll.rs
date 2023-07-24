use crate::integration::common::corelib::{corelib, predeployed_contracts};
use crate::integration::common::runner::Contract;
use crate::{assert_passed, test_case};
use camino::Utf8PathBuf;
use forge::run;
use indoc::indoc;
use std::path::Path;

#[test]
fn start_roll_simple() {
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
            trait IRollChecker<TContractState> {
                fn is_rolled(ref self: TContractState, expected_block_number: u64);
            }

            #[test]
            fn test_roll_simple() {
                let class_hash = declare('RollChecker').unwrap();
                let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @ArrayTrait::new() };
                let contract_address = deploy(prepared).unwrap();
                let contract_address: ContractAddress = contract_address.try_into().unwrap();
                let dispatcher = IRollCheckerDispatcher { contract_address };
            
                start_roll(contract_address, 234);
            
                dispatcher.is_rolled(234);
            }
        "#
        ),
        Contract::from_code_path(
            "RollChecker".to_string(),
            Path::new("tests/data/contracts/roll_checker.cairo"),
        )
        .unwrap()
    );

    let result = run(
        &test.path().unwrap(),
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
fn start_roll_in_constructor_test() {
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
            trait IConstructorRollChecker<TContractState> {
                fn was_rolled_on_construction(ref self: TContractState, expected_block_number: u64);
            }
            
            #[test]
            fn test_roll_constructor_simple() {
                assert(1 == 1, 'simple check');
                let class_hash = declare('ConstructorRollChecker').unwrap();
                let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @ArrayTrait::new() };
                let contract_address: ContractAddress = 2598896470772924212281968896271340780432065735045468431712403008297614014532.try_into().unwrap();
                start_roll(contract_address, 234);
                let contract_address: ContractAddress = deploy(prepared).unwrap().try_into().unwrap();
                contract_address.print();
            
                let dispatcher = IConstructorRollCheckerDispatcher { contract_address };
                dispatcher.was_rolled_on_construction(234);
            }
        "#
        ),
        Contract::from_code_path(
            "ConstructorRollChecker".to_string(),
            Path::new("tests/data/contracts/constructor_roll_checker.cairo"),
        )
        .unwrap()
    );

    let result = run(
        &test.path().unwrap(),
        &Some(test.linked_libraries()),
        &Default::default(),
        Some(&Utf8PathBuf::from_path_buf(corelib().to_path_buf()).unwrap()),
        &test.contracts(corelib().path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();

    assert_passed!(result);
}
