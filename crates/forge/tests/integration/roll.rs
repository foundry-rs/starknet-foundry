use crate::integration::common::corelib::{corelib_path, predeployed_contracts};
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
            use cheatcodes::{ declare, PreparedContract, deploy, start_roll };

            #[starknet::interface]
            trait IRollChecker<TContractState> {
                fn get_block_number(ref self: TContractState) -> u64;
            }

            #[test]
            fn test_roll_simple() {
                let class_hash = declare('RollChecker');
                let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @ArrayTrait::new() };
                let contract_address = deploy(prepared).unwrap();
                let dispatcher = IRollCheckerDispatcher { contract_address };

                start_roll(contract_address, 234);

                let block_number = dispatcher.get_block_number();
                assert(block_number == 234, 'Wrong block number');
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
fn start_roll_with_other_syscall() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use cheatcodes::{ declare, PreparedContract, deploy, start_roll };
            
            #[starknet::interface]
            trait IRollChecker<TContractState> {
                fn get_block_number_and_emit_event(ref self: TContractState) -> u64;
            }

            #[test]
            fn test_roll_simple() {
                let class_hash = declare('RollChecker');
                let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @ArrayTrait::new() };
                let contract_address = deploy(prepared).unwrap();
                let dispatcher = IRollCheckerDispatcher { contract_address };

                start_roll(contract_address, 234);

                let block_number = dispatcher.get_block_number_and_emit_event();
                assert(block_number == 234, 'Wrong block number');
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
            use cheatcodes::{ declare, PreparedContract, deploy, start_roll };

            #[starknet::interface]
            trait IConstructorRollChecker<TContractState> {
                fn get_stored_block_number(ref self: TContractState) -> u64;
            }

            #[test]
            fn test_roll_constructor_simple() {
                let class_hash = declare('ConstructorRollChecker');
                let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @ArrayTrait::new() };
                let contract_address: ContractAddress = 2598896470772924212281968896271340780432065735045468431712403008297614014532.try_into().unwrap();
                start_roll(contract_address, 234);
                let contract_address = deploy(prepared).unwrap();

                let dispatcher = IConstructorRollCheckerDispatcher { contract_address };
                assert(dispatcher.get_stored_block_number() == 234, 'Wrong stored blk_nb');
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
fn stop_roll() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use cheatcodes::{ declare, PreparedContract, deploy, start_roll, stop_roll };

            #[starknet::interface]
            trait IRollChecker<TContractState> {
                fn get_block_number(ref self: TContractState) -> u64;
            }

            #[test]
            fn test_stop_roll() {
                let class_hash = declare('RollChecker');
                let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @ArrayTrait::new() };
                let contract_address = deploy(prepared).unwrap();
                let dispatcher = IRollCheckerDispatcher { contract_address };

                let old_block_number = dispatcher.get_block_number();

                start_roll(contract_address, 234);

                let new_block_number = dispatcher.get_block_number();
                assert(new_block_number == 234, 'Wrong block number');

                stop_roll(contract_address);

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
fn double_roll() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use cheatcodes::{ declare, PreparedContract, deploy, start_roll, stop_roll };

            #[starknet::interface]
            trait IRollChecker<TContractState> {
                fn get_block_number(ref self: TContractState) -> u64;
            }

            #[test]
            fn test_stop_roll() {
                let class_hash = declare('RollChecker');
                let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @ArrayTrait::new() };
                let contract_address = deploy(prepared).unwrap();
                let dispatcher = IRollCheckerDispatcher { contract_address };

                let old_block_number = dispatcher.get_block_number();

                start_roll(contract_address, 234);
                start_roll(contract_address, 234);

                let new_block_number = dispatcher.get_block_number();
                assert(new_block_number == 234, 'Wrong block number');

                stop_roll(contract_address);

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
fn start_roll_with_proxy() {
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
            use cheatcodes::{ declare, PreparedContract, deploy, start_roll };
            
            #[starknet::interface]
            trait IRollCheckerProxy<TContractState> {
                fn get_roll_checkers_block_info(ref self: TContractState, address: ContractAddress) -> u64;
            }
            #[test]
            fn test_roll_simple() {
                let class_hash = declare('RollChecker');
                let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @ArrayTrait::new() };
                let roll_checker_contract_address = deploy(prepared).unwrap();
                start_roll(roll_checker_contract_address, 234);

                let class_hash = declare('RollCheckerProxy');
                let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @ArrayTrait::new() };
                let proxy_contract_address = deploy(prepared).unwrap();
                let proxy_dispatcher = IRollCheckerProxyDispatcher { contract_address: proxy_contract_address };
                let block_number = proxy_dispatcher.get_roll_checkers_block_info(roll_checker_contract_address);
                assert(block_number == 234, block_number.into());
            }
        "#
        ),
        Contract::from_code_path(
            "RollChecker".to_string(),
            Path::new("tests/data/contracts/roll_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "RollCheckerProxy".to_string(),
            Path::new("tests/data/contracts/roll_checker_proxy.cairo"),
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
fn start_roll_with_library_call() {
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
            use cheatcodes::{ declare, PreparedContract, deploy, start_roll };
            use starknet::ClassHash;

            #[starknet::interface]
            trait IRollCheckerLibCall<TContractState> {
                fn get_block_number_with_lib_call(ref self: TContractState, class_hash: ClassHash) -> u64;
            }

            #[test]
            fn test_roll_simple() {
                let roll_checker_class_hash = declare('RollChecker');

                let class_hash = declare('RollCheckerLibCall');
                let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @ArrayTrait::new() };
                let contract_address = deploy(prepared).unwrap();

                start_roll(contract_address, 234);

                let dispatcher = IRollCheckerLibCallDispatcher { contract_address };
                let block_number = dispatcher.get_block_number_with_lib_call(roll_checker_class_hash);
                assert(block_number == 234, block_number.into());
            }
        "#
        ),
        Contract::from_code_path(
            "RollChecker".to_string(),
            Path::new("tests/data/contracts/roll_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "RollCheckerLibCall".to_string(),
            Path::new("tests/data/contracts/roll_checker_library_call.cairo"),
        )
        .unwrap()
    );

    let result = run(
        &test.path().unwrap(),
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
