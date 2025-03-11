use indoc::indoc;
use std::path::Path;
use test_utils::runner::{Contract, assert_passed};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
fn cheat_block_hash_basic() {
    let test = test_case!(
        indoc!(
            r#"
            use starknet::get_block_info;
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait, cheat_block_hash };

            #[starknet::interface]
            trait ICheatBlockHashChecker<TContractState> {
                fn get_block_hash(ref self: TContractState) -> felt252;
                fn get_block_hash_and_number(ref self: TContractState) -> (felt252, u64);
            }
            fn deploy_cheat_block_hash_checker()  -> ICheatBlockHashCheckerDispatcher {
                let contract = declare("CheatBlockHashChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                ICheatBlockHashCheckerDispatcher { contract_address }
            }

            #[test]
            fn test_cheat_block_hash() {
                let cheat_block_hash_checker = deploy_cheat_block_hash_checker();
                let (old_block_hash, block_number) = cheat_block_hash_checker.get_block_hash_and_number();
                cheat_block_hash(block_number, 125);
                let new_block_hash = cheat_block_hash_checker.get_block_hash();
                assert(new_block_hash == 125, 'Wrong block hash');
                cheat_block_hash(block_number, old_block_hash);
                let new_block_hash = cheat_block_hash_checker.get_block_hash();
                assert(new_block_hash == old_block_hash, 'hash did not change back')
            }

            #[test]
            fn cheat_block_hash_multiple() {
                let contract = declare("CheatBlockHashChecker").unwrap().contract_class();
                let (contract_address1, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let (contract_address2, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let cheat_block_hash_checker1 = ICheatBlockHashCheckerDispatcher { contract_address: contract_address1 };
                let cheat_block_hash_checker2 = ICheatBlockHashCheckerDispatcher { contract_address: contract_address2 };
                let block_number = get_block_info().unbox().block_number - 10;
                let old_block_hash1 = cheat_block_hash_checker1.get_block_hash();
                let old_block_hash2 = cheat_block_hash_checker2.get_block_hash();
                cheat_block_hash(block_number, 123);
                let new_block_hash1 = cheat_block_hash_checker1.get_block_hash();
                let new_block_hash2 = cheat_block_hash_checker2.get_block_hash();
                assert(new_block_hash1 == 123, 'Wrong block hash #1');
                assert(new_block_hash2 == 123, 'Wrong block hash #2');
                cheat_block_hash(block_number, old_block_hash1);
                let new_block_hash1 = cheat_block_hash_checker1.get_block_hash();
                let new_block_hash2 = cheat_block_hash_checker2.get_block_hash();
                assert(new_block_hash1 == old_block_hash1, 'not stopped #1');
                assert(new_block_hash2 == old_block_hash2, 'not stopped #2');
            }
        "#
        ),
        Contract::from_code_path(
            "CheatBlockHashChecker".to_string(),
            Path::new("tests/data/contracts/cheat_block_hash_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}
