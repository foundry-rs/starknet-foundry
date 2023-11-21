use indoc::indoc;
use std::path::Path;
use test_utils::runner::Contract;
use test_utils::running_tests::run_test_case;
use test_utils::{assert_passed, test_case};

#[test]
fn elect() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ declare, ContractClassTrait, start_elect, stop_elect };

            #[starknet::interface]
            trait IElectChecker<TContractState> {
                fn get_sequencer_address(ref self: TContractState) -> ContractAddress;
            }

            #[test]
            fn test_stop_elect() {
                let contract = declare('ElectChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IElectCheckerDispatcher { contract_address };

                let old_sequencer_address = dispatcher.get_sequencer_address();

                start_elect(contract_address, 234.try_into().unwrap());

                let new_sequencer_address = dispatcher.get_sequencer_address();
                assert(new_sequencer_address == 234.try_into().unwrap(), 'Wrong sequencer address');

                stop_elect(contract_address);

                let new_sequencer_address = dispatcher.get_sequencer_address();
                assert(new_sequencer_address == old_sequencer_address, 'Sequencer addr did not revert');
            }
        "#
        ),
        Contract::from_code_path(
            "ElectChecker".to_string(),
            Path::new("tests/data/contracts/elect_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}
