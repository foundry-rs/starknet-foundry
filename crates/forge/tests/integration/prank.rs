use indoc::indoc;
use std::path::Path;
use utils::runner::Contract;
use utils::running_tests::run_test_case;
use utils::{assert_passed, test_case};

#[test]
fn prank() {
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
