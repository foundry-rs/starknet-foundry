use indoc::indoc;
use std::path::Path;
use test_utils::runner::{Contract, assert_passed};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
fn cheat_sequencer_address_basic() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait, start_cheat_sequencer_address, start_cheat_sequencer_address_global, stop_cheat_sequencer_address_global, stop_cheat_sequencer_address };

            #[starknet::interface]
            trait ICheatSequencerAddressChecker<TContractState> {
                fn get_sequencer_address(ref self: TContractState) -> ContractAddress;
            }

            #[test]
            fn test_stop_cheat_sequencer_address() {
                let contract = declare("CheatSequencerAddressChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ICheatSequencerAddressCheckerDispatcher { contract_address };

                let old_sequencer_address = dispatcher.get_sequencer_address();

                start_cheat_sequencer_address(contract_address, 234.try_into().unwrap());

                let new_sequencer_address = dispatcher.get_sequencer_address();
                assert(new_sequencer_address == 234.try_into().unwrap(), 'Wrong sequencer address');

                stop_cheat_sequencer_address(contract_address);

                let new_sequencer_address = dispatcher.get_sequencer_address();
                assert(new_sequencer_address == old_sequencer_address, 'Sequencer addr did not revert');
            }

            #[test]
            fn test_cheat_sequencer_address_multiple() {
                let contract = declare("CheatSequencerAddressChecker").unwrap().contract_class();

                let (contract_address1, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let (contract_address2, _) = contract.deploy(@ArrayTrait::new()).unwrap();

                let cheat_sequencer_address_checker1 = ICheatSequencerAddressCheckerDispatcher { contract_address: contract_address1 };
                let cheat_sequencer_address_checker2 = ICheatSequencerAddressCheckerDispatcher { contract_address: contract_address2 };

                let old_seq_addr1 = cheat_sequencer_address_checker1.get_sequencer_address();
                let old_seq_addr2 = cheat_sequencer_address_checker2.get_sequencer_address();

                start_cheat_sequencer_address(cheat_sequencer_address_checker1.contract_address, 123.try_into().unwrap());
                start_cheat_sequencer_address(cheat_sequencer_address_checker2.contract_address, 123.try_into().unwrap());

                let new_seq_addr1 = cheat_sequencer_address_checker1.get_sequencer_address();
                let new_seq_addr2 = cheat_sequencer_address_checker2.get_sequencer_address();

                assert(new_seq_addr1 == 123.try_into().unwrap(), 'Wrong seq addr #1');
                assert(new_seq_addr2 == 123.try_into().unwrap(), 'Wrong seq addr #2');

                stop_cheat_sequencer_address(cheat_sequencer_address_checker1.contract_address);
                stop_cheat_sequencer_address(cheat_sequencer_address_checker2.contract_address);

                let new_seq_addr1 = cheat_sequencer_address_checker1.get_sequencer_address();
                let new_seq_addr2 = cheat_sequencer_address_checker2.get_sequencer_address();

                assert(new_seq_addr1 == old_seq_addr1, 'not stopped #1');
                assert(new_seq_addr2 == old_seq_addr2, 'not stopped #2');
            }
            #[test]
            fn test_cheat_sequencer_address_all() {
                let contract = declare("CheatSequencerAddressChecker").unwrap().contract_class();

                let (contract_address1, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let (contract_address2, _) = contract.deploy(@ArrayTrait::new()).unwrap();

                let cheat_sequencer_address_checker1 = ICheatSequencerAddressCheckerDispatcher { contract_address: contract_address1 };
                let cheat_sequencer_address_checker2 = ICheatSequencerAddressCheckerDispatcher { contract_address: contract_address2 };

                let old_seq_addr1 = cheat_sequencer_address_checker1.get_sequencer_address();
                let old_seq_addr2 = cheat_sequencer_address_checker2.get_sequencer_address();

                start_cheat_sequencer_address_global(123.try_into().unwrap());

                let new_seq_addr1 = cheat_sequencer_address_checker1.get_sequencer_address();
                let new_seq_addr2 = cheat_sequencer_address_checker2.get_sequencer_address();

                assert(new_seq_addr1 == 123.try_into().unwrap(), 'Wrong seq addr #1');
                assert(new_seq_addr2 == 123.try_into().unwrap(), 'Wrong seq addr #2');

                stop_cheat_sequencer_address_global();

                let new_seq_addr1 = cheat_sequencer_address_checker1.get_sequencer_address();
                let new_seq_addr2 = cheat_sequencer_address_checker2.get_sequencer_address();

                assert(new_seq_addr1 == old_seq_addr1, 'Wrong seq addr #1');
                assert(new_seq_addr2 == old_seq_addr2, 'Wrong seq addr #2');
            }

            #[test]
            fn test_cheat_sequencer_address_all_stop_one() {
                let contract = declare("CheatSequencerAddressChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ICheatSequencerAddressCheckerDispatcher { contract_address };

                let target_seq_addr: felt252 = 123;
                let target_seq_addr: ContractAddress = target_seq_addr.try_into().unwrap();

                let old_seq_addr = dispatcher.get_sequencer_address();

                start_cheat_sequencer_address_global(target_seq_addr);

                let new_seq_addr = dispatcher.get_sequencer_address();
                assert(new_seq_addr == 123.try_into().unwrap(), 'Wrong seq addr');

                stop_cheat_sequencer_address(contract_address);

                let new_seq_addr = dispatcher.get_sequencer_address();
                assert(old_seq_addr == new_seq_addr, 'Address did not change back');
            }
        "#
        ),
        Contract::from_code_path(
            "CheatSequencerAddressChecker".to_string(),
            Path::new("tests/data/contracts/cheat_sequencer_address_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn cheat_sequencer_address_complex() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait, start_cheat_sequencer_address, start_cheat_sequencer_address_global, stop_cheat_sequencer_address };

            #[starknet::interface]
            trait ICheatSequencerAddressChecker<TContractState> {
                fn get_sequencer_address(ref self: TContractState) -> ContractAddress;
            }

            #[test]
            fn test_cheat_sequencer_address_complex() {
                let contract = declare("CheatSequencerAddressChecker").unwrap().contract_class();

                let (contract_address1, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let (contract_address2, _) = contract.deploy(@ArrayTrait::new()).unwrap();

                let cheat_sequencer_address_checker1 = ICheatSequencerAddressCheckerDispatcher { contract_address: contract_address1 };
                let cheat_sequencer_address_checker2 = ICheatSequencerAddressCheckerDispatcher { contract_address: contract_address2 };

                let old_seq_addr2 = cheat_sequencer_address_checker2.get_sequencer_address();

                start_cheat_sequencer_address_global(123.try_into().unwrap());

                let new_seq_addr1 = cheat_sequencer_address_checker1.get_sequencer_address();
                let new_seq_addr2 = cheat_sequencer_address_checker2.get_sequencer_address();

                assert(new_seq_addr1 == 123.try_into().unwrap(), 'Wrong seq addr #1');
                assert(new_seq_addr2 == 123.try_into().unwrap(), 'Wrong seq addr #2');

                start_cheat_sequencer_address(cheat_sequencer_address_checker1.contract_address, 456.try_into().unwrap());

                let new_seq_addr1 = cheat_sequencer_address_checker1.get_sequencer_address();
                let new_seq_addr2 = cheat_sequencer_address_checker2.get_sequencer_address();

                assert(new_seq_addr1 == 456.try_into().unwrap(), 'Wrong seq addr #3');
                assert(new_seq_addr2 == 123.try_into().unwrap(), 'Wrong seq addr #4');

                start_cheat_sequencer_address(cheat_sequencer_address_checker1.contract_address, 789.try_into().unwrap());
                start_cheat_sequencer_address(cheat_sequencer_address_checker2.contract_address, 789.try_into().unwrap());

                let new_seq_addr1 = cheat_sequencer_address_checker1.get_sequencer_address();
                let new_seq_addr2 = cheat_sequencer_address_checker2.get_sequencer_address();

                assert(new_seq_addr1 == 789.try_into().unwrap(), 'Wrong seq addr #5');
                assert(new_seq_addr2 == 789.try_into().unwrap(), 'Wrong seq addr #6');

                stop_cheat_sequencer_address(cheat_sequencer_address_checker2.contract_address);

                let new_seq_addr1 = cheat_sequencer_address_checker1.get_sequencer_address();
                let new_seq_addr2 = cheat_sequencer_address_checker2.get_sequencer_address();

                assert(new_seq_addr1 == 789.try_into().unwrap(), 'Wrong seq addr #7');
                assert(new_seq_addr2 == old_seq_addr2, 'Wrong seq addr #8');
            }
        "#
        ),
        Contract::from_code_path(
            "CheatSequencerAddressChecker".to_string(),
            Path::new("tests/data/contracts/cheat_sequencer_address_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn cheat_sequencer_address_with_span() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ test_address, declare, ContractClassTrait, DeclareResultTrait, cheat_sequencer_address, start_cheat_sequencer_address, stop_cheat_sequencer_address, CheatSpan };

            #[starknet::interface]
            trait ICheatSequencerAddressChecker<TContractState> {
                fn get_sequencer_address(ref self: TContractState) -> felt252;
            }

            fn deploy_cheat_sequencer_address_checker() -> ICheatSequencerAddressCheckerDispatcher {
                let (contract_address, _) = declare("CheatSequencerAddressChecker").unwrap().contract_class().deploy(@ArrayTrait::new()).unwrap();
                ICheatSequencerAddressCheckerDispatcher { contract_address }
            }

            #[test]
            fn test_cheat_sequencer_address_once() {
                let old_sequencer_address = get_sequencer_address();

                let dispatcher = deploy_cheat_sequencer_address_checker();

                let target_sequencer_address: ContractAddress = 123.try_into().unwrap();

                cheat_sequencer_address(dispatcher.contract_address, target_sequencer_address, CheatSpan::TargetCalls(1));

                let sequencer_address = dispatcher.get_sequencer_address();
                assert(sequencer_address == target_sequencer_address.into(), 'Wrong sequencer address');

                let sequencer_address = dispatcher.get_sequencer_address();
                assert(sequencer_address == old_sequencer_address.into(), 'Address did not change back');
            }

            #[test]
            fn test_cheat_sequencer_address_twice() {
                let old_sequencer_address = get_sequencer_address();

                let dispatcher = deploy_cheat_sequencer_address_checker();

                let target_sequencer_address: ContractAddress = 123.try_into().unwrap();

                cheat_sequencer_address(dispatcher.contract_address, target_sequencer_address, CheatSpan::TargetCalls(2));

                let sequencer_address = dispatcher.get_sequencer_address();
                assert(sequencer_address == target_sequencer_address.into(), 'Wrong sequencer address');

                let sequencer_address = dispatcher.get_sequencer_address();
                assert(sequencer_address == target_sequencer_address.into(), 'Wrong sequencer address');

                let sequencer_address = dispatcher.get_sequencer_address();
                assert(sequencer_address == old_sequencer_address.into(), 'Address did not change back');
            }

            #[test]
            fn test_cheat_sequencer_address_test_address() {
                let old_sequencer_address = get_sequencer_address();

                let target_sequencer_address: ContractAddress = 123.try_into().unwrap();

                cheat_sequencer_address(test_address(), target_sequencer_address, CheatSpan::TargetCalls(1));

                let sequencer_address = get_sequencer_address();
                assert(sequencer_address == target_sequencer_address, 'Wrong sequencer address');

                let sequencer_address = get_sequencer_address();
                assert(sequencer_address == target_sequencer_address, 'Wrong sequencer address');

                stop_cheat_sequencer_address(test_address());

                let sequencer_address = get_sequencer_address();
                assert(sequencer_address == old_sequencer_address, 'Wrong sequencer address');
            }

            fn get_sequencer_address() -> ContractAddress {
                starknet::get_block_info().unbox().sequencer_address
            }
        "#
        ),
        Contract::from_code_path(
            "CheatSequencerAddressChecker".to_string(),
            Path::new("tests/data/contracts/cheat_sequencer_address_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}
