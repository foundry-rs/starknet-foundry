use indoc::indoc;
use std::path::Path;
use test_utils::runner::{assert_passed, Contract};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
fn cheat_block_hash_basic() {
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
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait, start_cheat_block_hash, stop_cheat_block_hash, start_cheat_block_number, start_cheat_block_hash_global, stop_cheat_block_hash_global };

            #[starknet::interface]
            trait ICheatBlockhashChecker<TContractState> {
                fn get_block_hash(ref self: TContractState) -> felt252;
                fn get_block_hash_and_emit_event(ref self: TContractState) -> felt252;
                fn get_block_hash_and_number(ref self: TContractState) -> (felt252, u64);
            }

            fn deploy_cheat_block_hash_checker()  -> ICheatBlockhashCheckerDispatcher {
                let contract = declare("CheatBlockhashChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                ICheatBlockhashCheckerDispatcher { contract_address }
            }

            #[test]
            fn test_cheat_block_hash() {
                let cheat_block_hash_checker = deploy_cheat_block_hash_checker();

                let old_block_hash = cheat_block_hash_checker.get_block_hash();

                start_cheat_block_hash(cheat_block_hash_checker.contract_address, 123);

                let new_block_hash = cheat_block_hash_checker.get_block_hash();
                assert(new_block_hash == 123, 'Wrong block hash');

                stop_cheat_block_hash(cheat_block_hash_checker.contract_address);

                let new_block_hash = cheat_block_hash_checker.get_block_hash();
                assert(new_block_hash == old_block_hash, 'hash did not change back')
            }

            #[test]
            fn cheat_block_hash_all_stop_one() {
                let cheat_block_hash_checker = deploy_cheat_block_hash_checker();

                let old_block_hash = cheat_block_hash_checker.get_block_hash();

                start_cheat_block_hash_global(123);

                let new_block_hash = cheat_block_hash_checker.get_block_hash();
                assert(new_block_hash == 123, 'Wrong block hash');

                stop_cheat_block_hash(cheat_block_hash_checker.contract_address);

                let new_block_hash = cheat_block_hash_checker.get_block_hash();
                assert(new_block_hash == old_block_hash, 'hash did not change back')
            }

            #[test]
            fn cheat_block_hash_multiple() {
                let contract = declare("CheatBlockhashChecker").unwrap().contract_class();

                let (contract_address1, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let (contract_address2, _) = contract.deploy(@ArrayTrait::new()).unwrap();

                let cheat_block_hash_checker1 = ICheatBlockhashCheckerDispatcher { contract_address: contract_address1 };
                let cheat_block_hash_checker2 = ICheatBlockhashCheckerDispatcher { contract_address: contract_address2 };

                let old_block_hash1 = cheat_block_hash_checker1.get_block_hash();
                let old_block_hash2 = cheat_block_hash_checker2.get_block_hash();

                start_cheat_block_hash(cheat_block_hash_checker1.contract_address, 123);
                start_cheat_block_hash(cheat_block_hash_checker2.contract_address, 123);

                let new_block_hash1 = cheat_block_hash_checker1.get_block_hash();
                let new_block_hash2 = cheat_block_hash_checker2.get_block_hash();

                assert(new_block_hash1 == 123, 'Wrong block hash #1');
                assert(new_block_hash2 == 123, 'Wrong block hash #2');

                stop_cheat_block_hash(cheat_block_hash_checker1.contract_address);
                stop_cheat_block_hash(cheat_block_hash_checker2.contract_address);

                let new_block_hash1 = cheat_block_hash_checker1.get_block_hash();
                let new_block_hash2 = cheat_block_hash_checker2.get_block_hash();

                assert(new_block_hash1 == old_block_hash1, 'not stopped #1');
                assert(new_block_hash2 == old_block_hash2, 'not stopped #2');
            }

            #[test]
            fn cheat_block_hash_all() {
                let contract = declare("CheatBlockhashChecker").unwrap().contract_class();

                let (contract_address1, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let (contract_address2, _) = contract.deploy(@ArrayTrait::new()).unwrap();

                let cheat_block_hash_checker1 = ICheatBlockhashCheckerDispatcher { contract_address: contract_address1 };
                let cheat_block_hash_checker2 = ICheatBlockhashCheckerDispatcher { contract_address: contract_address2 };

                let old_block_hash1 = cheat_block_hash_checker1.get_block_hash();
                let old_block_hash2 = cheat_block_hash_checker2.get_block_hash();

                start_cheat_block_hash_global(123);

                let new_block_hash1 = cheat_block_hash_checker1.get_block_hash();
                let new_block_hash2 = cheat_block_hash_checker2.get_block_hash();

                assert(new_block_hash1 == 123, 'Wrong block hash #1');
                assert(new_block_hash2 == 123, 'Wrong block hash #2');

                stop_cheat_block_hash_global();

                let new_block_hash1 = cheat_block_hash_checker1.get_block_hash();
                let new_block_hash2 = cheat_block_hash_checker2.get_block_hash();

                assert(new_block_hash1 == old_block_hash1, 'Wrong block hash #1');
                assert(new_block_hash2 == old_block_hash2, 'Wrong block hash #2');
            }
        "#
        ),
        Contract::from_code_path(
            "CheatBlockhashChecker".to_string(),
            Path::new("tests/data/contracts/cheat_block_hash_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn cheat_block_hash_complex() {
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
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait, start_cheat_block_hash, stop_cheat_block_hash, start_cheat_block_number, start_cheat_block_hash_global };

            #[starknet::interface]
            trait ICheatBlockhashChecker<TContractState> {
                fn get_block_hash(ref self: TContractState) -> felt252;
                fn get_block_hash_and_emit_event(ref self: TContractState) -> felt252;
                fn get_block_hash_and_number(ref self: TContractState) -> (felt252, u64);
            }

            fn deploy_cheat_block_hash_checker()  -> ICheatBlockhashCheckerDispatcher {
                let contract = declare("CheatBlockhashChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                ICheatBlockhashCheckerDispatcher { contract_address }
            }

            #[test]
            fn cheat_block_hash_complex() {
                let contract = declare("CheatBlockhashChecker").unwrap().contract_class();

                let (contract_address1, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let (contract_address2, _) = contract.deploy(@ArrayTrait::new()).unwrap();

                let cheat_block_hash_checker1 = ICheatBlockhashCheckerDispatcher { contract_address: contract_address1 };
                let cheat_block_hash_checker2 = ICheatBlockhashCheckerDispatcher { contract_address: contract_address2 };

                let old_block_hash2 = cheat_block_hash_checker2.get_block_hash();

                start_cheat_block_hash_global(123);

                let new_block_hash1 = cheat_block_hash_checker1.get_block_hash();
                let new_block_hash2 = cheat_block_hash_checker2.get_block_hash();

                assert(new_block_hash1 == 123, 'Wrong block hash #1');
                assert(new_block_hash2 == 123, 'Wrong block hash #2');

                start_cheat_block_hash(cheat_block_hash_checker1.contract_address, 456);

                let new_block_hash1 = cheat_block_hash_checker1.get_block_hash();
                let new_block_hash2 = cheat_block_hash_checker2.get_block_hash();

                assert(new_block_hash1 == 456, 'Wrong block hash #3');
                assert(new_block_hash2 == 123, 'Wrong block hash #4');

                start_cheat_block_hash(cheat_block_hash_checker1.contract_address, 789);
                start_cheat_block_hash(cheat_block_hash_checker2.contract_address, 789);

                let new_block_hash1 = cheat_block_hash_checker1.get_block_hash();
                let new_block_hash2 = cheat_block_hash_checker2.get_block_hash();

                assert(new_block_hash1 == 789, 'Wrong block hash #5');
                assert(new_block_hash2 == 789, 'Wrong block hash #6');

                stop_cheat_block_hash(cheat_block_hash_checker2.contract_address);

                let new_block_hash1 = cheat_block_hash_checker1.get_block_hash();
                let new_block_hash2 = cheat_block_hash_checker2.get_block_hash();

                assert(new_block_hash1 == 789, 'Wrong block hash #7');
                assert(new_block_hash2 == old_block_hash2, 'Wrong block hash #8');
            }
        "#
        ),
        Contract::from_code_path(
            "CheatBlockhashChecker".to_string(),
            Path::new("tests/data/contracts/cheat_block_hash_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn cheat_block_hash_with_span() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{ test_address, declare, ContractClassTrait, DeclareResultTrait, cheat_block_hash, start_cheat_block_hash, stop_cheat_block_hash, CheatSpan };

            #[starknet::interface]
            trait ICheatBlockhashChecker<TContractState> {
                fn get_block_hash(ref self: TContractState) -> felt252;
            }

            fn deploy_cheat_block_hash_checker() -> ICheatBlockhashCheckerDispatcher {
                let (contract_address, _) = declare("CheatBlockhashChecker").unwrap().contract_class().deploy(@ArrayTrait::new()).unwrap();
                ICheatBlockhashCheckerDispatcher { contract_address }
            }

            #[test]
            fn test_cheat_block_hash_once() {
                let old_block_hash = get_block_hash();

                let dispatcher = deploy_cheat_block_hash_checker();

                let target_block_hash = 123;

                cheat_block_hash(dispatcher.contract_address, target_block_hash, CheatSpan::TargetCalls(1));

                let block_hash = dispatcher.get_block_hash();
                assert_eq!(block_hash, target_block_hash.into());

                let block_hash = dispatcher.get_block_hash();
                assert_eq!(block_hash, old_block_hash.into());
            }

            #[test]
            fn test_cheat_block_hash_twice() {
                let old_block_hash = get_block_hash();

                let dispatcher = deploy_cheat_block_hash_checker();

                let target_block_hash = 123;

                cheat_block_hash(dispatcher.contract_address, target_block_hash, CheatSpan::TargetCalls(2));

                let block_hash = dispatcher.get_block_hash();
                assert_eq!(block_hash, target_block_hash.into());

                let block_hash = dispatcher.get_block_hash();
                assert_eq!(block_hash, target_block_hash.into());

                let block_hash = dispatcher.get_block_hash();
                assert_eq!(block_hash, old_block_hash.into());
            }

            #[test]
            fn test_cheat_block_hash_test_address() {
                let old_block_hash = get_block_hash();

                let target_block_hash = 123;

                cheat_block_hash(test_address(), target_block_hash, CheatSpan::TargetCalls(1));

                let block_hash = get_block_hash();
                assert_eq!(block_hash, target_block_hash.into());

                let block_hash = get_block_hash();
                assert_eq!(block_hash, target_block_hash.into());

                stop_cheat_block_hash(test_address());

                let block_hash = get_block_hash();
                assert_eq!(block_hash, old_block_hash.into());
            }

            fn get_block_hash() -> felt252 {
                starknet::get_block_info().unbox().block_hash
            }
        "#
        ),
        Contract::from_code_path(
            "CheatBlockhashChecker".to_string(),
            Path::new("tests/data/contracts/cheat_block_hash_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}
