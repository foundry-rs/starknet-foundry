use forge_runner::forge_config::ForgeTrackedResource;
use indoc::indoc;
use std::path::Path;
use test_utils::runner::{Contract, assert_passed};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
fn cheat_block_timestamp_basic() {
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
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait, start_cheat_block_timestamp, stop_cheat_block_timestamp, start_cheat_block_number, start_cheat_block_timestamp_global, stop_cheat_block_timestamp_global };

            #[starknet::interface]
            trait ICheatBlockTimestampChecker<TContractState> {
                fn get_block_timestamp(ref self: TContractState) -> u64;
                fn get_block_timestamp_and_emit_event(ref self: TContractState) -> u64;
                fn get_block_timestamp_and_number(ref self: TContractState) -> (u64, u64);
            }

            fn deploy_cheat_block_timestamp_checker()  -> ICheatBlockTimestampCheckerDispatcher {
                let contract = declare("CheatBlockTimestampChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                ICheatBlockTimestampCheckerDispatcher { contract_address }
            }

            #[test]
            fn test_cheat_block_timestamp() {
                let cheat_block_timestamp_checker = deploy_cheat_block_timestamp_checker();

                let old_block_timestamp = cheat_block_timestamp_checker.get_block_timestamp();

                start_cheat_block_timestamp(cheat_block_timestamp_checker.contract_address, 123);

                let new_block_timestamp = cheat_block_timestamp_checker.get_block_timestamp();
                assert(new_block_timestamp == 123, 'Wrong block timestamp');

                stop_cheat_block_timestamp(cheat_block_timestamp_checker.contract_address);

                let new_block_timestamp = cheat_block_timestamp_checker.get_block_timestamp();
                assert(new_block_timestamp == old_block_timestamp, 'Timestamp did not change back')
            }

            #[test]
            fn cheat_block_timestamp_all_stop_one() {
                let cheat_block_timestamp_checker = deploy_cheat_block_timestamp_checker();

                let old_block_timestamp = cheat_block_timestamp_checker.get_block_timestamp();

                start_cheat_block_timestamp_global(123);

                let new_block_timestamp = cheat_block_timestamp_checker.get_block_timestamp();
                assert(new_block_timestamp == 123, 'Wrong block timestamp');

                stop_cheat_block_timestamp(cheat_block_timestamp_checker.contract_address);

                let new_block_timestamp = cheat_block_timestamp_checker.get_block_timestamp();
                assert(new_block_timestamp == old_block_timestamp, 'Timestamp did not change back')
            }

            #[test]
            fn cheat_block_timestamp_multiple() {
                let contract = declare("CheatBlockTimestampChecker").unwrap().contract_class();

                let (contract_address1, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let (contract_address2, _) = contract.deploy(@ArrayTrait::new()).unwrap();

                let cheat_block_timestamp_checker1 = ICheatBlockTimestampCheckerDispatcher { contract_address: contract_address1 };
                let cheat_block_timestamp_checker2 = ICheatBlockTimestampCheckerDispatcher { contract_address: contract_address2 };

                let old_block_timestamp1 = cheat_block_timestamp_checker1.get_block_timestamp();
                let old_block_timestamp2 = cheat_block_timestamp_checker2.get_block_timestamp();

                start_cheat_block_timestamp(cheat_block_timestamp_checker1.contract_address, 123);
                start_cheat_block_timestamp(cheat_block_timestamp_checker2.contract_address, 123);

                let new_block_timestamp1 = cheat_block_timestamp_checker1.get_block_timestamp();
                let new_block_timestamp2 = cheat_block_timestamp_checker2.get_block_timestamp();

                assert(new_block_timestamp1 == 123, 'Wrong block timestamp #1');
                assert(new_block_timestamp2 == 123, 'Wrong block timestamp #2');

                stop_cheat_block_timestamp(cheat_block_timestamp_checker1.contract_address);
                stop_cheat_block_timestamp(cheat_block_timestamp_checker2.contract_address);

                let new_block_timestamp1 = cheat_block_timestamp_checker1.get_block_timestamp();
                let new_block_timestamp2 = cheat_block_timestamp_checker2.get_block_timestamp();

                assert(new_block_timestamp1 == old_block_timestamp1, 'not stopped #1');
                assert(new_block_timestamp2 == old_block_timestamp2, 'not stopped #2');
            }

            #[test]
            fn cheat_block_timestamp_all() {
                let contract = declare("CheatBlockTimestampChecker").unwrap().contract_class();

                let (contract_address1, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let (contract_address2, _) = contract.deploy(@ArrayTrait::new()).unwrap();

                let cheat_block_timestamp_checker1 = ICheatBlockTimestampCheckerDispatcher { contract_address: contract_address1 };
                let cheat_block_timestamp_checker2 = ICheatBlockTimestampCheckerDispatcher { contract_address: contract_address2 };

                let old_block_timestamp1 = cheat_block_timestamp_checker1.get_block_timestamp();
                let old_block_timestamp2 = cheat_block_timestamp_checker2.get_block_timestamp();

                start_cheat_block_timestamp_global(123);

                let new_block_timestamp1 = cheat_block_timestamp_checker1.get_block_timestamp();
                let new_block_timestamp2 = cheat_block_timestamp_checker2.get_block_timestamp();

                assert(new_block_timestamp1 == 123, 'Wrong block timestamp #1');
                assert(new_block_timestamp2 == 123, 'Wrong block timestamp #2');

                stop_cheat_block_timestamp_global();

                let new_block_timestamp1 = cheat_block_timestamp_checker1.get_block_timestamp();
                let new_block_timestamp2 = cheat_block_timestamp_checker2.get_block_timestamp();

                assert(new_block_timestamp1 == old_block_timestamp1, 'Wrong block timestamp #1');
                assert(new_block_timestamp2 == old_block_timestamp2, 'Wrong block timestamp #2');
            }
        "#
        ),
        Contract::from_code_path(
            "CheatBlockTimestampChecker".to_string(),
            Path::new("tests/data/contracts/cheat_block_timestamp_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn cheat_block_timestamp_complex() {
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
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait, start_cheat_block_timestamp, stop_cheat_block_timestamp, start_cheat_block_number, start_cheat_block_timestamp_global };

            #[starknet::interface]
            trait ICheatBlockTimestampChecker<TContractState> {
                fn get_block_timestamp(ref self: TContractState) -> u64;
                fn get_block_timestamp_and_emit_event(ref self: TContractState) -> u64;
                fn get_block_timestamp_and_number(ref self: TContractState) -> (u64, u64);
            }

            fn deploy_cheat_block_timestamp_checker()  -> ICheatBlockTimestampCheckerDispatcher {
                let contract = declare("CheatBlockTimestampChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                ICheatBlockTimestampCheckerDispatcher { contract_address }
            }

            #[test]
            fn cheat_block_timestamp_complex() {
                let contract = declare("CheatBlockTimestampChecker").unwrap().contract_class();

                let (contract_address1, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let (contract_address2, _) = contract.deploy(@ArrayTrait::new()).unwrap();

                let cheat_block_timestamp_checker1 = ICheatBlockTimestampCheckerDispatcher { contract_address: contract_address1 };
                let cheat_block_timestamp_checker2 = ICheatBlockTimestampCheckerDispatcher { contract_address: contract_address2 };

                let old_block_timestamp2 = cheat_block_timestamp_checker2.get_block_timestamp();

                start_cheat_block_timestamp_global(123);

                let new_block_timestamp1 = cheat_block_timestamp_checker1.get_block_timestamp();
                let new_block_timestamp2 = cheat_block_timestamp_checker2.get_block_timestamp();

                assert(new_block_timestamp1 == 123, 'Wrong block timestamp #1');
                assert(new_block_timestamp2 == 123, 'Wrong block timestamp #2');

                start_cheat_block_timestamp(cheat_block_timestamp_checker1.contract_address, 456);

                let new_block_timestamp1 = cheat_block_timestamp_checker1.get_block_timestamp();
                let new_block_timestamp2 = cheat_block_timestamp_checker2.get_block_timestamp();

                assert(new_block_timestamp1 == 456, 'Wrong block timestamp #3');
                assert(new_block_timestamp2 == 123, 'Wrong block timestamp #4');

                start_cheat_block_timestamp(cheat_block_timestamp_checker1.contract_address, 789);
                start_cheat_block_timestamp(cheat_block_timestamp_checker2.contract_address, 789);

                let new_block_timestamp1 = cheat_block_timestamp_checker1.get_block_timestamp();
                let new_block_timestamp2 = cheat_block_timestamp_checker2.get_block_timestamp();

                assert(new_block_timestamp1 == 789, 'Wrong block timestamp #5');
                assert(new_block_timestamp2 == 789, 'Wrong block timestamp #6');

                stop_cheat_block_timestamp(cheat_block_timestamp_checker2.contract_address);

                let new_block_timestamp1 = cheat_block_timestamp_checker1.get_block_timestamp();
                let new_block_timestamp2 = cheat_block_timestamp_checker2.get_block_timestamp();

                assert(new_block_timestamp1 == 789, 'Wrong block timestamp #7');
                assert(new_block_timestamp2 == old_block_timestamp2, 'Wrong block timestamp #8');
            }
        "#
        ),
        Contract::from_code_path(
            "CheatBlockTimestampChecker".to_string(),
            Path::new("tests/data/contracts/cheat_block_timestamp_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn cheat_block_timestamp_with_span() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ test_address, declare, ContractClassTrait, DeclareResultTrait, cheat_block_timestamp, start_cheat_block_timestamp, stop_cheat_block_timestamp, CheatSpan };

            #[starknet::interface]
            trait ICheatBlockTimestampChecker<TContractState> {
                fn get_block_timestamp(ref self: TContractState) -> felt252;
            }

            fn deploy_cheat_block_timestamp_checker() -> ICheatBlockTimestampCheckerDispatcher {
                let (contract_address, _) = declare("CheatBlockTimestampChecker").unwrap().contract_class().deploy(@ArrayTrait::new()).unwrap();
                ICheatBlockTimestampCheckerDispatcher { contract_address }
            }

            #[test]
            fn test_cheat_block_timestamp_once() {
                let old_block_timestamp = get_block_timestamp();

                let dispatcher = deploy_cheat_block_timestamp_checker();

                let target_block_timestamp = 123;

                cheat_block_timestamp(dispatcher.contract_address, target_block_timestamp, CheatSpan::TargetCalls(1));

                let block_timestamp = dispatcher.get_block_timestamp();
                assert_eq!(block_timestamp, target_block_timestamp.into());

                let block_timestamp = dispatcher.get_block_timestamp();
                assert_eq!(block_timestamp, old_block_timestamp.into());
            }

            #[test]
            fn test_cheat_block_timestamp_twice() {
                let old_block_timestamp = get_block_timestamp();

                let dispatcher = deploy_cheat_block_timestamp_checker();

                let target_block_timestamp = 123;

                cheat_block_timestamp(dispatcher.contract_address, target_block_timestamp, CheatSpan::TargetCalls(2));

                let block_timestamp = dispatcher.get_block_timestamp();
                assert_eq!(block_timestamp, target_block_timestamp.into());

                let block_timestamp = dispatcher.get_block_timestamp();
                assert_eq!(block_timestamp, target_block_timestamp.into());

                let block_timestamp = dispatcher.get_block_timestamp();
                assert_eq!(block_timestamp, old_block_timestamp.into());
            }

            #[test]
            fn test_cheat_block_timestamp_test_address() {
                let old_block_timestamp = get_block_timestamp();

                let target_block_timestamp = 123;

                cheat_block_timestamp(test_address(), target_block_timestamp, CheatSpan::TargetCalls(1));

                let block_timestamp = get_block_timestamp();
                assert_eq!(block_timestamp, target_block_timestamp.into());

                let block_timestamp = get_block_timestamp();
                assert_eq!(block_timestamp, target_block_timestamp.into());

                stop_cheat_block_timestamp(test_address());

                let block_timestamp = get_block_timestamp();
                assert_eq!(block_timestamp, old_block_timestamp.into());
            }

            fn get_block_timestamp() -> u64 {
                starknet::get_block_info().unbox().block_timestamp
            }
        "#
        ),
        Contract::from_code_path(
            "CheatBlockTimestampChecker".to_string(),
            Path::new("tests/data/contracts/cheat_block_timestamp_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}
