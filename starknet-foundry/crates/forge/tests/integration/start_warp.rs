use crate::integration::common::corelib::{corelib, predeployed_contracts};
use crate::integration::common::runner::Contract;
use crate::{assert_passed, test_case};
use camino::Utf8PathBuf;
use forge::run;
use indoc::indoc;
use std::path::Path;

#[test]
fn start_warp_simple() {
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
            use cheatcodes::PreparedContract;
            
            #[starknet::interface]
            trait IWarpChecker<TContractState> {
                fn get_block_timestamp(ref self: TContractState) -> u64;
                fn get_block_timestamp_and_emit_event(ref self: TContractState) -> u64;
                fn get_block_timestamp_and_number(ref self: TContractState) -> (u64, u64);
            }
            
            fn deploy_warp_checker()  -> IWarpCheckerDispatcher {
                let class_hash = declare('WarpChecker').unwrap();
                let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @ArrayTrait::new() };
                let contract_address = deploy(prepared).unwrap();
                let contract_address: ContractAddress = contract_address.try_into().unwrap();
                IWarpCheckerDispatcher { contract_address }                
            }

            #[test]
            fn test_warp_simple() {
                let warp_checker = deploy_warp_checker();
                start_warp(warp_checker.contract_address, 234);
            
                let block_timestamp = warp_checker.get_block_timestamp();
                assert(block_timestamp == 234, block_timestamp.into());
            }
            
            #[test]
            fn test_warp_with_emit() {
                let warp_checker = deploy_warp_checker();
                start_warp(warp_checker.contract_address, 234);
            
                let block_timestamp = warp_checker.get_block_timestamp_and_emit_event();
                assert(block_timestamp == 234, 'Wrong block timestamp');
            }
            
            #[test]
            fn test_warp_with_roll() {
                let warp_checker = deploy_warp_checker();
                start_warp(warp_checker.contract_address, 123);
                start_roll(warp_checker.contract_address, 456);
                
                let (block_timestamp, block_number) = warp_checker.get_block_timestamp_and_number();
                assert(block_timestamp == 123, 'Wrong block timestamp');
                assert(block_number == 456, 'Wrong block number');
            }
        "#
        ),
        Contract::from_code_path(
            "WarpChecker".to_string(),
            Path::new("tests/data/contracts/warp_checker.cairo"),
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
fn start_warp_in_constructor_test() {
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
            trait IConstructorWarpChecker<TContractState> {
                fn get_stored_block_timestamp(ref self: TContractState) -> u64;
            }
            
            #[test]
            fn test_warp_constructor_simple() {
                let class_hash = declare('ConstructorWarpChecker').unwrap();
                let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @ArrayTrait::new() };
                let contract_address: ContractAddress = 3536868843103376321721783970179672615412806578951102081876401371045020950704.try_into().unwrap();
                start_roll(contract_address, 234);
                let contract_address: ContractAddress = deploy(prepared).unwrap().try_into().unwrap();
            
                let dispatcher = IConstructorWarpCheckerDispatcher { contract_address };
                assert(dispatcher.get_stored_block_timestamp() == 234, 'Wrong stored timestamp');
            }
        "#
        ),
        Contract::from_code_path(
            "ConstructorWarpChecker".to_string(),
            Path::new("tests/data/contracts/constructor_warp_checker.cairo"),
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
