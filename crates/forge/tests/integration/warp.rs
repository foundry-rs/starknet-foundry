use indoc::indoc;
use std::path::Path;
use test_utils::runner::{assert_passed, Contract};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
fn warp_basic() {
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
            use snforge_std::{ declare, CheatTarget, ContractClassTrait, start_warp, stop_warp, start_roll };

            #[starknet::interface]
            trait IWarpChecker<TContractState> {
                fn get_block_timestamp(ref self: TContractState) -> u64;
                fn get_block_timestamp_and_emit_event(ref self: TContractState) -> u64;
                fn get_block_timestamp_and_number(ref self: TContractState) -> (u64, u64);
            }

            fn deploy_warp_checker()  -> IWarpCheckerDispatcher {
                let contract = declare("WarpChecker").unwrap();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                IWarpCheckerDispatcher { contract_address }
            }

            #[test]
            fn test_warp() {
                let warp_checker = deploy_warp_checker();

                let old_block_timestamp = warp_checker.get_block_timestamp();

                start_warp(CheatTarget::One(warp_checker.contract_address), 123);

                let new_block_timestamp = warp_checker.get_block_timestamp();
                assert(new_block_timestamp == 123, 'Wrong block timestamp');

                stop_warp(CheatTarget::One(warp_checker.contract_address));

                let new_block_timestamp = warp_checker.get_block_timestamp();
                assert(new_block_timestamp == old_block_timestamp, 'Timestamp did not change back')
            }

            #[test]
            fn warp_all_stop_one() {
                let warp_checker = deploy_warp_checker();

                let old_block_timestamp = warp_checker.get_block_timestamp();

                start_warp(CheatTarget::All, 123);

                let new_block_timestamp = warp_checker.get_block_timestamp();
                assert(new_block_timestamp == 123, 'Wrong block timestamp');

                stop_warp(CheatTarget::One(warp_checker.contract_address));

                let new_block_timestamp = warp_checker.get_block_timestamp();
                assert(new_block_timestamp == old_block_timestamp, 'Timestamp did not change back')
            }

            #[test]
            fn warp_multiple() {
                let contract = declare("WarpChecker").unwrap();

                let (contract_address1, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let (contract_address2, _) = contract.deploy(@ArrayTrait::new()).unwrap();

                let warp_checker1 = IWarpCheckerDispatcher { contract_address: contract_address1 };
                let warp_checker2 = IWarpCheckerDispatcher { contract_address: contract_address2 };

                let old_block_timestamp1 = warp_checker1.get_block_timestamp();
                let old_block_timestamp2 = warp_checker2.get_block_timestamp();

                start_warp(CheatTarget::Multiple(array![warp_checker1.contract_address, warp_checker2.contract_address]), 123);

                let new_block_timestamp1 = warp_checker1.get_block_timestamp();
                let new_block_timestamp2 = warp_checker2.get_block_timestamp();

                assert(new_block_timestamp1 == 123, 'Wrong block timestamp #1');
                assert(new_block_timestamp2 == 123, 'Wrong block timestamp #2');

                stop_warp(CheatTarget::Multiple(array![warp_checker1.contract_address, warp_checker2.contract_address]));

                let new_block_timestamp1 = warp_checker1.get_block_timestamp();
                let new_block_timestamp2 = warp_checker2.get_block_timestamp();

                assert(new_block_timestamp1 == old_block_timestamp1, 'Warp not stopped #1');
                assert(new_block_timestamp2 == old_block_timestamp2, 'Warp not stopped #2');
            }

            #[test]
            fn warp_all() {
                let contract = declare("WarpChecker").unwrap();

                let (contract_address1, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let (contract_address2, _) = contract.deploy(@ArrayTrait::new()).unwrap();

                let warp_checker1 = IWarpCheckerDispatcher { contract_address: contract_address1 };
                let warp_checker2 = IWarpCheckerDispatcher { contract_address: contract_address2 };

                let old_block_timestamp1 = warp_checker1.get_block_timestamp();
                let old_block_timestamp2 = warp_checker2.get_block_timestamp();

                start_warp(CheatTarget::All, 123);

                let new_block_timestamp1 = warp_checker1.get_block_timestamp();
                let new_block_timestamp2 = warp_checker2.get_block_timestamp();

                assert(new_block_timestamp1 == 123, 'Wrong block timestamp #1');
                assert(new_block_timestamp2 == 123, 'Wrong block timestamp #2');

                stop_warp(CheatTarget::All);

                let new_block_timestamp1 = warp_checker1.get_block_timestamp();
                let new_block_timestamp2 = warp_checker2.get_block_timestamp();

                assert(new_block_timestamp1 == old_block_timestamp1, 'Warp not stopped #1');
                assert(new_block_timestamp2 == old_block_timestamp2, 'Warp not stopped #2');
            }
        "#
        ),
        Contract::from_code_path(
            "WarpChecker".to_string(),
            Path::new("tests/data/contracts/warp_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn warp_complex() {
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
            use snforge_std::{ declare, CheatTarget, ContractClassTrait, start_warp, stop_warp, start_roll };

            #[starknet::interface]
            trait IWarpChecker<TContractState> {
                fn get_block_timestamp(ref self: TContractState) -> u64;
                fn get_block_timestamp_and_emit_event(ref self: TContractState) -> u64;
                fn get_block_timestamp_and_number(ref self: TContractState) -> (u64, u64);
            }

            fn deploy_warp_checker()  -> IWarpCheckerDispatcher {
                let contract = declare("WarpChecker").unwrap();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                IWarpCheckerDispatcher { contract_address }
            }

            #[test]
            fn warp_complex() {
                let contract = declare("WarpChecker").unwrap();

                let (contract_address1, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let (contract_address2, _) = contract.deploy(@ArrayTrait::new()).unwrap();

                let warp_checker1 = IWarpCheckerDispatcher { contract_address: contract_address1 };
                let warp_checker2 = IWarpCheckerDispatcher { contract_address: contract_address2 };

                let old_block_timestamp2 = warp_checker2.get_block_timestamp();

                start_warp(CheatTarget::All, 123);

                let new_block_timestamp1 = warp_checker1.get_block_timestamp();
                let new_block_timestamp2 = warp_checker2.get_block_timestamp();
                
                assert(new_block_timestamp1 == 123, 'Wrong block timestamp #1');
                assert(new_block_timestamp2 == 123, 'Wrong block timestamp #2');

                start_warp(CheatTarget::One(warp_checker1.contract_address), 456);

                let new_block_timestamp1 = warp_checker1.get_block_timestamp();
                let new_block_timestamp2 = warp_checker2.get_block_timestamp();

                assert(new_block_timestamp1 == 456, 'Wrong block timestamp #3');
                assert(new_block_timestamp2 == 123, 'Wrong block timestamp #4');

                start_warp(CheatTarget::Multiple(array![warp_checker1.contract_address, warp_checker2.contract_address]), 789);

                let new_block_timestamp1 = warp_checker1.get_block_timestamp();
                let new_block_timestamp2 = warp_checker2.get_block_timestamp();

                assert(new_block_timestamp1 == 789, 'Wrong block timestamp #5');
                assert(new_block_timestamp2 == 789, 'Wrong block timestamp #6');

                stop_warp(CheatTarget::One(warp_checker2.contract_address));

                let new_block_timestamp1 = warp_checker1.get_block_timestamp();
                let new_block_timestamp2 = warp_checker2.get_block_timestamp();

                assert(new_block_timestamp1 == 789, 'Wrong block timestamp #7');
                assert(new_block_timestamp2 == old_block_timestamp2, 'Wrong block timestamp #8');
            }
        "#
        ),
        Contract::from_code_path(
            "WarpChecker".to_string(),
            Path::new("tests/data/contracts/warp_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn warp_with_span() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ test_address, declare, ContractClassTrait, warp, start_warp, stop_warp, CheatTarget, CheatSpan };

            #[starknet::interface]
            trait IWarpChecker<TContractState> {
                fn get_block_timestamp(ref self: TContractState) -> felt252;
            }

            fn deploy_warp_checker() -> IWarpCheckerDispatcher {
                let (contract_address, _) = declare("WarpChecker").unwrap().deploy(@ArrayTrait::new()).unwrap();
                IWarpCheckerDispatcher { contract_address }
            }

            #[test]
            fn test_warp_once() {
                let old_block_timestamp = get_block_timestamp();
                
                let dispatcher = deploy_warp_checker();

                let target_block_timestamp = 123;

                warp(CheatTarget::One(dispatcher.contract_address), target_block_timestamp, CheatSpan::TargetCalls(1));

                let block_timestamp = dispatcher.get_block_timestamp();
                assert_eq!(block_timestamp, target_block_timestamp.into());

                let block_timestamp = dispatcher.get_block_timestamp();
                assert_eq!(block_timestamp, old_block_timestamp.into());
            }

            #[test]
            fn test_warp_twice() {
                let old_block_timestamp = get_block_timestamp();

                let dispatcher = deploy_warp_checker();

                let target_block_timestamp = 123;

                warp(CheatTarget::One(dispatcher.contract_address), target_block_timestamp, CheatSpan::TargetCalls(2));

                let block_timestamp = dispatcher.get_block_timestamp();
                assert_eq!(block_timestamp, target_block_timestamp.into());
                
                let block_timestamp = dispatcher.get_block_timestamp();
                assert_eq!(block_timestamp, target_block_timestamp.into());

                let block_timestamp = dispatcher.get_block_timestamp();
                assert_eq!(block_timestamp, old_block_timestamp.into());
            }

            #[test]
            fn test_warp_test_address() {
                let old_block_timestamp = get_block_timestamp();
                
                let target_block_timestamp = 123;
                
                warp(CheatTarget::One(test_address()), target_block_timestamp, CheatSpan::TargetCalls(1));
                
                let block_timestamp = get_block_timestamp();
                assert_eq!(block_timestamp, target_block_timestamp.into());

                let block_timestamp = get_block_timestamp();
                assert_eq!(block_timestamp, target_block_timestamp.into());
                
                stop_warp(CheatTarget::One(test_address()));

                let block_timestamp = get_block_timestamp();
                assert_eq!(block_timestamp, old_block_timestamp.into());
            }

            fn get_block_timestamp() -> u64 {
                starknet::get_block_info().unbox().block_timestamp
            }
        "#
        ),
        Contract::from_code_path(
            "WarpChecker".to_string(),
            Path::new("tests/data/contracts/warp_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}
