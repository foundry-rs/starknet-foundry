use crate::integration::common::runner::Contract;
use crate::integration::common::running_tests::run_test_case;
use crate::{assert_passed, test_case};
use indoc::indoc;
use std::path::Path;

#[test]
fn get_class_hash() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use snforge_std::{ declare, ContractClassTrait, start_prank, get_class_hash };

            #[test]
            fn test_get_class_hash() {
                let contract = declare('GetClassHashCheckerUpg');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                assert(get_class_hash(contract_address) == contract.class_hash, 'Incorrect class hash');
            }
        "#
        ),
        Contract::from_code_path(
            "GetClassHashCheckerUpg".to_string(),
            Path::new("tests/data/contracts/get_class_hash_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn get_class_hash_replace_class() {
    let test = test_case!(
        indoc!(
            r#"
            use array::{ArrayTrait, SpanTrait};
            use core::result::ResultTrait;
            use starknet::ClassHash;
            use snforge_std::{ declare, ContractClassTrait, get_class_hash };

            #[starknet::interface]
            trait IUpgradeable<T> {
                fn upgrade(ref self: T, class_hash: ClassHash);
            }

            #[starknet::interface]
            trait IHelloStarknet<TContractState> {
                fn increase_balance(ref self: TContractState, amount: felt252);
                fn get_balance(self: @TContractState) -> felt252;
                fn do_a_panic(self: @TContractState);
                fn do_a_panic_with(self: @TContractState, panic_data: Array<felt252>);
            }

            #[test]
            fn test_get_class_hash_replace_class() {
                let contract = declare('GetClassHashCheckerUpg');

                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();

                assert(get_class_hash(contract_address) == contract.class_hash, 'Incorrect class hash');

                let hsn_contract= declare('HelloStarknet');
                IUpgradeableDispatcher { contract_address }.upgrade(hsn_contract.class_hash);
                assert(get_class_hash(contract_address) == hsn_contract.class_hash, 'Incorrect upgrade class hash');

                let hello_dispatcher = IHelloStarknetDispatcher { contract_address };
                hello_dispatcher.increase_balance(42);
                assert(hello_dispatcher.get_balance() == 42, 'Invalid balance');
            }
        "#
        ),
        Contract::from_code_path(
            "GetClassHashCheckerUpg".to_string(),
            Path::new("tests/data/contracts/get_class_hash_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "HelloStarknet".to_string(),
            Path::new("tests/data/contracts/hello_starknet.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}
