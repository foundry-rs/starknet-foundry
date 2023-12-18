use indoc::indoc;
use std::path::Path;
use test_utils::runner::Contract;
use test_utils::running_tests::run_test_case;
use test_utils::{assert_passed, test_case};

#[test]
fn elect_basic() {
    let test = test_case!(
        indoc!(
            r"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ declare, CheatTarget, ContractClassTrait, start_elect, stop_elect };

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

                start_elect(CheatTarget::One(contract_address), 234.try_into().unwrap());

                let new_sequencer_address = dispatcher.get_sequencer_address();
                assert(new_sequencer_address == 234.try_into().unwrap(), 'Wrong sequencer address');

                stop_elect(CheatTarget::One(contract_address));

                let new_sequencer_address = dispatcher.get_sequencer_address();
                assert(new_sequencer_address == old_sequencer_address, 'Sequencer addr did not revert');
            }

            #[test]
            fn test_elect_multiple() {
                let contract = declare('ElectChecker');

                let elect_checker1 = IElectCheckerDispatcher { contract_address: contract.deploy(@ArrayTrait::new()).unwrap() };
                let elect_checker2 = IElectCheckerDispatcher { contract_address: contract.deploy(@ArrayTrait::new()).unwrap() };

                let old_seq_addr1 = elect_checker1.get_sequencer_address();
                let old_seq_addr2 = elect_checker2.get_sequencer_address();

                start_elect(CheatTarget::Multiple(array![elect_checker1.contract_address, elect_checker2.contract_address]), 123.try_into().unwrap());

                let new_seq_addr1 = elect_checker1.get_sequencer_address();
                let new_seq_addr2 = elect_checker2.get_sequencer_address();

                assert(new_seq_addr1 == 123.try_into().unwrap(), 'Wrong seq addr #1');
                assert(new_seq_addr2 == 123.try_into().unwrap(), 'Wrong seq addr #2');

                stop_elect(CheatTarget::Multiple(array![elect_checker1.contract_address, elect_checker2.contract_address]));

                let new_seq_addr1 = elect_checker1.get_sequencer_address();
                let new_seq_addr2 = elect_checker2.get_sequencer_address();

                assert(new_seq_addr1 == old_seq_addr1, 'Elect not stopped #1');
                assert(new_seq_addr2 == old_seq_addr2, 'Elect not stopped #2');
            }
            #[test]
            fn test_elect_all() {
                let contract = declare('ElectChecker');

                let elect_checker1 = IElectCheckerDispatcher { contract_address: contract.deploy(@ArrayTrait::new()).unwrap() };
                let elect_checker2 = IElectCheckerDispatcher { contract_address: contract.deploy(@ArrayTrait::new()).unwrap() };

                let old_seq_addr1 = elect_checker1.get_sequencer_address();
                let old_seq_addr2 = elect_checker2.get_sequencer_address();

                start_elect(CheatTarget::All, 123.try_into().unwrap());

                let new_seq_addr1 = elect_checker1.get_sequencer_address();
                let new_seq_addr2 = elect_checker2.get_sequencer_address();

                assert(new_seq_addr1 == 123.try_into().unwrap(), 'Wrong seq addr #1');
                assert(new_seq_addr2 == 123.try_into().unwrap(), 'Wrong seq addr #2');

                stop_elect(CheatTarget::All);

                let new_seq_addr1 = elect_checker1.get_sequencer_address();
                let new_seq_addr2 = elect_checker2.get_sequencer_address();

                assert(new_seq_addr1 == old_seq_addr1, 'Elect not stopped #1');
                assert(new_seq_addr2 == old_seq_addr2, 'Elect not stopped #2');
            }

            #[test]
            fn test_elect_all_stop_one() {
                let contract = declare('ElectChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IElectCheckerDispatcher { contract_address };

                let target_seq_addr: felt252 = 123;
                let target_seq_addr: ContractAddress = target_seq_addr.try_into().unwrap();

                let old_seq_addr = dispatcher.get_sequencer_address();

                start_elect(CheatTarget::All, target_seq_addr);

                let new_seq_addr = dispatcher.get_sequencer_address();
                assert(new_seq_addr == 123.try_into().unwrap(), 'Wrong seq addr');

                stop_elect(CheatTarget::One(contract_address));

                let new_seq_addr = dispatcher.get_sequencer_address();
                assert(old_seq_addr == new_seq_addr, 'Address did not change back');
            }
        "
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

#[test]
fn elect_complex() {
    let test = test_case!(
        indoc!(
            r"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ declare, CheatTarget, ContractClassTrait, start_elect, stop_elect };
            
            #[starknet::interface]
            trait IElectChecker<TContractState> {
                fn get_sequencer_address(ref self: TContractState) -> ContractAddress;
            }

            #[test]
            fn test_elect_complex() {
                let contract = declare('ElectChecker');

                let elect_checker1 = IElectCheckerDispatcher { contract_address: contract.deploy(@ArrayTrait::new()).unwrap() };
                let elect_checker2 = IElectCheckerDispatcher { contract_address: contract.deploy(@ArrayTrait::new()).unwrap() };

                let old_seq_addr2 = elect_checker2.get_sequencer_address();

                start_elect(CheatTarget::All, 123.try_into().unwrap());

                let new_seq_addr1 = elect_checker1.get_sequencer_address();
                let new_seq_addr2 = elect_checker2.get_sequencer_address();
                
                assert(new_seq_addr1 == 123.try_into().unwrap(), 'Wrong seq addr #1');
                assert(new_seq_addr2 == 123.try_into().unwrap(), 'Wrong seq addr #2');

                start_elect(CheatTarget::One(elect_checker1.contract_address), 456.try_into().unwrap());

                let new_seq_addr1 = elect_checker1.get_sequencer_address();
                let new_seq_addr2 = elect_checker2.get_sequencer_address();

                assert(new_seq_addr1 == 456.try_into().unwrap(), 'Wrong seq addr #3');
                assert(new_seq_addr2 == 123.try_into().unwrap(), 'Wrong seq addr #4');

                start_elect(CheatTarget::Multiple(array![elect_checker1.contract_address, elect_checker2.contract_address]), 789.try_into().unwrap());

                let new_seq_addr1 = elect_checker1.get_sequencer_address();
                let new_seq_addr2 = elect_checker2.get_sequencer_address();

                assert(new_seq_addr1 == 789.try_into().unwrap(), 'Wrong seq addr #5');
                assert(new_seq_addr2 == 789.try_into().unwrap(), 'Wrong seq addr #6');

                stop_elect(CheatTarget::One(elect_checker2.contract_address));

                let new_seq_addr1 = elect_checker1.get_sequencer_address();
                let new_seq_addr2 = elect_checker2.get_sequencer_address();
                
                assert(new_seq_addr1 == 789.try_into().unwrap(), 'Wrong seq addr #7');
                assert(new_seq_addr2 == old_seq_addr2, 'Wrong seq addr #8');
            }
        "
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
