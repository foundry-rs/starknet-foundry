use indoc::indoc;
use std::path::Path;
use utils::runner::Contract;
use utils::running_tests::run_test_case;
use utils::{assert_passed, test_case};

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
            use snforge_std::{ declare, ContractClassTrait, start_warp, stop_warp, start_roll };

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
            fn test_warp() {
                let warp_checker = deploy_warp_checker();

                let old_block_timestamp = warp_checker.get_block_timestamp();

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

    let result = run_test_case(&test);

    assert_passed!(result);
}
