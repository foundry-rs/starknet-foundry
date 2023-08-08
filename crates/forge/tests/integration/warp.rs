use crate::integration::common::corelib::{corelib_path, predeployed_contracts};
use crate::integration::common::runner::Contract;
use crate::{assert_passed, test_case};
use camino::Utf8PathBuf;
use forge::run;
use indoc::indoc;
use std::path::Path;

#[test]
fn warp() {
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
            use cheatcodes::{ declare, ContractClassTrait, start_warp, stop_warp, start_roll };

            #[starknet::interface]
            trait IWarpChecker<TContractState> {
                fn get_block_timestamp(ref self: TContractState) -> u64;
                fn get_block_timestamp_and_emit_event(ref self: TContractState) -> u64;
                fn get_block_timestamp_and_number(ref self: TContractState) -> (u64, u64);
            }

            fn deploy_warp_checker()  -> IWarpCheckerDispatcher {
                let contract = declare('WarpChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
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

            #[test]
            fn test_stop_warp() {
                let warp_checker = deploy_warp_checker();

                let old_block_timestamp = warp_checker.get_block_timestamp();

                start_warp(warp_checker.contract_address, 123);

                let new_block_timestamp = warp_checker.get_block_timestamp();
                assert(new_block_timestamp == 123, 'Wrong block timestamp');

                stop_warp(warp_checker.contract_address);

                let new_block_timestamp = warp_checker.get_block_timestamp();
                assert(new_block_timestamp == old_block_timestamp, 'Timestamp did not change back')
            }

            #[test]
            fn test_double_warp() {
                let warp_checker = deploy_warp_checker();

                let old_block_timestamp = warp_checker.get_block_timestamp();

                start_warp(warp_checker.contract_address, 123);
                start_warp(warp_checker.contract_address, 123);

                let new_block_timestamp = warp_checker.get_block_timestamp();
                assert(new_block_timestamp == 123, 'Wrong block timestamp');

                stop_warp(warp_checker.contract_address);

                let new_block_timestamp = warp_checker.get_block_timestamp();
                assert(new_block_timestamp == old_block_timestamp, 'Timestamp did not change back')
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
        &String::from("src"),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
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
            use cheatcodes::{ declare, PreparedContract, deploy, start_warp };
            
            #[starknet::interface]
            trait IConstructorWarpChecker<TContractState> {
                fn get_stored_block_timestamp(ref self: TContractState) -> u64;
            }

            #[test]
            fn test_warp_constructor_simple() {
                let class_hash = declare('ConstructorWarpChecker');
                let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @ArrayTrait::new() };
                let contract_address: ContractAddress = 3536868843103376321721783970179672615412806578951102081876401371045020950704.try_into().unwrap();
                start_warp(contract_address, 234);
                let contract_address = deploy(prepared).unwrap().;            

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
        &String::from("src"),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();

    assert_passed!(result);
}

#[test]
fn start_warp_with_proxy() {
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
            use cheatcodes::{ declare, ContractClassTrait, start_warp };

            #[starknet::interface]
            trait IWarpCheckerProxy<TContractState> {
                fn get_warp_checkers_block_info(ref self: TContractState, address: ContractAddress) -> u64;
            }

            #[test]
            fn test_warp_simple() {
                let contract = declare('WarpChecker');
                let warp_checker_contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                start_warp(warp_checker_contract_address, 234);

                let contract = declare('WarpCheckerProxy');
                let proxy_contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let proxy_dispatcher = IWarpCheckerProxyDispatcher { contract_address: proxy_contract_address };

                let block_timestamp = proxy_dispatcher.get_warp_checkers_block_info(warp_checker_contract_address);
                assert(block_timestamp == 234, block_timestamp.into());
            }
        "#
        ),
        Contract::from_code_path(
            "WarpChecker".to_string(),
            Path::new("tests/data/contracts/warp_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "WarpCheckerProxy".to_string(),
            Path::new("tests/data/contracts/warp_checker_proxy.cairo"),
        )
        .unwrap()
    );

    let result = run(
        &test.path().unwrap(),
        &String::from("src"),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();

    assert_passed!(result);
}

#[test]
fn start_warp_with_library_call() {
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
            use cheatcodes::{ declare, ContractClassTrait, start_warp };
            use starknet::ClassHash;

            #[starknet::interface]
            trait IWarpCheckerLibCall<TContractState> {
                fn get_block_timestamp_with_lib_call(ref self: TContractState, class_hash: ClassHash) -> u64;
            }

            #[test]
            fn test_warp_simple() {
                let warp_checker_contract = declare('WarpChecker');

                let warp_checker_class_hash = warp_checker_contract.class_hash;

                let contract = declare('WarpCheckerLibCall');
                let contract_address = contract.deploy( @ArrayTrait::new()).unwrap();

                start_warp(contract_address, 234);

                let dispatcher = IWarpCheckerLibCallDispatcher { contract_address };
                let block_timestamp = dispatcher.get_block_timestamp_with_lib_call(warp_checker_class_hash);
                assert(block_timestamp == 234, block_timestamp.into());
            }
        "#
        ),
        Contract::from_code_path(
            "WarpChecker".to_string(),
            Path::new("tests/data/contracts/warp_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "WarpCheckerLibCall".to_string(),
            Path::new("tests/data/contracts/warp_checker_library_call.cairo"),
        )
        .unwrap()
    );

    let result = run(
        &test.path().unwrap(),
        &String::from("src"),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();

    assert_passed!(result);
}
