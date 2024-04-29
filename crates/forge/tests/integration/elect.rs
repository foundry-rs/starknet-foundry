use indoc::indoc;
use std::path::Path;
use test_utils::runner::{assert_passed, Contract};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
fn elect_basic() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ declare, ContractClassTrait, start_elect, elect_global, stop_elect_global, stop_elect };

            #[starknet::interface]
            trait IElectChecker<TContractState> {
                fn get_sequencer_address(ref self: TContractState) -> ContractAddress;
            }

            #[test]
            fn test_stop_elect() {
                let contract = declare("ElectChecker").unwrap();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IElectCheckerDispatcher { contract_address };

                let old_sequencer_address = dispatcher.get_sequencer_address();

                start_elect(contract_address, 234.try_into().unwrap());

                let new_sequencer_address = dispatcher.get_sequencer_address();
                assert(new_sequencer_address == 234.try_into().unwrap(), 'Wrong sequencer address');

                stop_elect(contract_address);

                let new_sequencer_address = dispatcher.get_sequencer_address();
                assert(new_sequencer_address == old_sequencer_address, 'Sequencer addr did not revert');
            }

            #[test]
            fn test_elect_multiple() {
                let contract = declare("ElectChecker").unwrap();

                let (contract_address1, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let (contract_address2, _) = contract.deploy(@ArrayTrait::new()).unwrap();

                let elect_checker1 = IElectCheckerDispatcher { contract_address: contract_address1 };
                let elect_checker2 = IElectCheckerDispatcher { contract_address: contract_address2 };

                let old_seq_addr1 = elect_checker1.get_sequencer_address();
                let old_seq_addr2 = elect_checker2.get_sequencer_address();

                start_elect(elect_checker1.contract_address, 123.try_into().unwrap());
                start_elect(elect_checker2.contract_address, 123.try_into().unwrap());

                let new_seq_addr1 = elect_checker1.get_sequencer_address();
                let new_seq_addr2 = elect_checker2.get_sequencer_address();

                assert(new_seq_addr1 == 123.try_into().unwrap(), 'Wrong seq addr #1');
                assert(new_seq_addr2 == 123.try_into().unwrap(), 'Wrong seq addr #2');

                stop_elect(elect_checker1.contract_address);
                stop_elect(elect_checker2.contract_address);

                let new_seq_addr1 = elect_checker1.get_sequencer_address();
                let new_seq_addr2 = elect_checker2.get_sequencer_address();

                assert(new_seq_addr1 == old_seq_addr1, 'Elect not stopped #1');
                assert(new_seq_addr2 == old_seq_addr2, 'Elect not stopped #2');
            }
            #[test]
            fn test_elect_all() {
                let contract = declare("ElectChecker").unwrap();

                let (contract_address1, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let (contract_address2, _) = contract.deploy(@ArrayTrait::new()).unwrap();

                let elect_checker1 = IElectCheckerDispatcher { contract_address: contract_address1 };
                let elect_checker2 = IElectCheckerDispatcher { contract_address: contract_address2 };

                elect_global(123.try_into().unwrap());

                let new_seq_addr1 = elect_checker1.get_sequencer_address();
                let new_seq_addr2 = elect_checker2.get_sequencer_address();

                assert(new_seq_addr1 == 123.try_into().unwrap(), 'Wrong seq addr #1');
                assert(new_seq_addr2 == 123.try_into().unwrap(), 'Wrong seq addr #2');

                stop_elect_global();

                let new_seq_addr1 = elect_checker1.get_sequencer_address();
                let new_seq_addr2 = elect_checker2.get_sequencer_address();

                assert(new_seq_addr1 == 123.try_into().unwrap(), 'Wrong seq addr #1');
                assert(new_seq_addr2 == 123.try_into().unwrap(), 'Wrong seq addr #2');
            }

            #[test]
            fn test_elect_all_stop_one() {
                let contract = declare("ElectChecker").unwrap();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IElectCheckerDispatcher { contract_address };

                let target_seq_addr: felt252 = 123;
                let target_seq_addr: ContractAddress = target_seq_addr.try_into().unwrap();

                let old_seq_addr = dispatcher.get_sequencer_address();

                elect_global(target_seq_addr);

                let new_seq_addr = dispatcher.get_sequencer_address();
                assert(new_seq_addr == 123.try_into().unwrap(), 'Wrong seq addr');

                stop_elect(contract_address);

                let new_seq_addr = dispatcher.get_sequencer_address();
                assert(old_seq_addr == new_seq_addr, 'Address did not change back');
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

    assert_passed(&result);
}

#[test]
fn elect_complex() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ declare, ContractClassTrait, start_elect, elect_global, stop_elect };
            
            #[starknet::interface]
            trait IElectChecker<TContractState> {
                fn get_sequencer_address(ref self: TContractState) -> ContractAddress;
            }

            #[test]
            fn test_elect_complex() {
                let contract = declare("ElectChecker").unwrap();

                let (contract_address1, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let (contract_address2, _) = contract.deploy(@ArrayTrait::new()).unwrap();

                let elect_checker1 = IElectCheckerDispatcher { contract_address: contract_address1 };
                let elect_checker2 = IElectCheckerDispatcher { contract_address: contract_address2 };

                let old_seq_addr2 = elect_checker2.get_sequencer_address();

                elect_global(123.try_into().unwrap());

                let new_seq_addr1 = elect_checker1.get_sequencer_address();
                let new_seq_addr2 = elect_checker2.get_sequencer_address();
                
                assert(new_seq_addr1 == 123.try_into().unwrap(), 'Wrong seq addr #1');
                assert(new_seq_addr2 == 123.try_into().unwrap(), 'Wrong seq addr #2');

                start_elect(elect_checker1.contract_address, 456.try_into().unwrap());

                let new_seq_addr1 = elect_checker1.get_sequencer_address();
                let new_seq_addr2 = elect_checker2.get_sequencer_address();

                assert(new_seq_addr1 == 456.try_into().unwrap(), 'Wrong seq addr #3');
                assert(new_seq_addr2 == 123.try_into().unwrap(), 'Wrong seq addr #4');

                start_elect(elect_checker1.contract_address, 789.try_into().unwrap());
                start_elect(elect_checker2.contract_address, 789.try_into().unwrap());

                let new_seq_addr1 = elect_checker1.get_sequencer_address();
                let new_seq_addr2 = elect_checker2.get_sequencer_address();

                assert(new_seq_addr1 == 789.try_into().unwrap(), 'Wrong seq addr #5');
                assert(new_seq_addr2 == 789.try_into().unwrap(), 'Wrong seq addr #6');

                stop_elect(elect_checker2.contract_address);

                let new_seq_addr1 = elect_checker1.get_sequencer_address();
                let new_seq_addr2 = elect_checker2.get_sequencer_address();
                
                assert(new_seq_addr1 == 789.try_into().unwrap(), 'Wrong seq addr #7');
                assert(new_seq_addr2 == old_seq_addr2, 'Wrong seq addr #8');
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

    assert_passed(&result);
}

#[test]
fn elect_with_span() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ test_address, declare, ContractClassTrait, elect, start_elect, stop_elect, CheatSpan };

            #[starknet::interface]
            trait IElectChecker<TContractState> {
                fn get_sequencer_address(ref self: TContractState) -> felt252;
            }

            fn deploy_elect_checker() -> IElectCheckerDispatcher {
                let (contract_address, _) = declare("ElectChecker").unwrap().deploy(@ArrayTrait::new()).unwrap();
                IElectCheckerDispatcher { contract_address }
            }

            #[test]
            fn test_elect_once() {
                let old_sequencer_address = get_sequencer_address();
                
                let dispatcher = deploy_elect_checker();

                let target_sequencer_address: ContractAddress = 123.try_into().unwrap();

                elect(dispatcher.contract_address, target_sequencer_address, CheatSpan::TargetCalls(1));

                let sequencer_address = dispatcher.get_sequencer_address();
                assert(sequencer_address == target_sequencer_address.into(), 'Wrong sequencer address');

                let sequencer_address = dispatcher.get_sequencer_address();
                assert(sequencer_address == old_sequencer_address.into(), 'Address did not change back');
            }

            #[test]
            fn test_elect_twice() {
                let old_sequencer_address = get_sequencer_address();

                let dispatcher = deploy_elect_checker();

                let target_sequencer_address: ContractAddress = 123.try_into().unwrap();

                elect(dispatcher.contract_address, target_sequencer_address, CheatSpan::TargetCalls(2));

                let sequencer_address = dispatcher.get_sequencer_address();
                assert(sequencer_address == target_sequencer_address.into(), 'Wrong sequencer address');
                
                let sequencer_address = dispatcher.get_sequencer_address();
                assert(sequencer_address == target_sequencer_address.into(), 'Wrong sequencer address');

                let sequencer_address = dispatcher.get_sequencer_address();
                assert(sequencer_address == old_sequencer_address.into(), 'Address did not change back');
            }

            #[test]
            fn test_elect_test_address() {
                let old_sequencer_address = get_sequencer_address();
                
                let target_sequencer_address: ContractAddress = 123.try_into().unwrap();
                
                elect(test_address(), target_sequencer_address, CheatSpan::TargetCalls(1));
                
                let sequencer_address = get_sequencer_address();
                assert(sequencer_address == target_sequencer_address, 'Wrong sequencer address');

                let sequencer_address = get_sequencer_address();
                assert(sequencer_address == target_sequencer_address, 'Wrong sequencer address');
                
                stop_elect(test_address());

                let sequencer_address = get_sequencer_address();
                assert(sequencer_address == old_sequencer_address, 'Wrong sequencer address');
            }

            fn get_sequencer_address() -> ContractAddress {
                starknet::get_block_info().unbox().sequencer_address
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

    assert_passed(&result);
}
