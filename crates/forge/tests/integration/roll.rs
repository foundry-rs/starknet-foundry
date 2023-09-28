use indoc::indoc;
use std::path::Path;
use utils::runner::Contract;
use utils::running_tests::run_test_case;
use utils::{assert_passed, test_case};

#[test]
fn roll() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ declare, ContractClassTrait, start_roll, stop_roll };

            #[starknet::interface]
            trait IRollChecker<TContractState> {
                fn get_block_number(ref self: TContractState) -> u64;
            }

            #[test]
            fn test_stop_roll() {
                let contract = declare('RollChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
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

    let result = run_test_case(&test);

    assert_passed!(result);
}
