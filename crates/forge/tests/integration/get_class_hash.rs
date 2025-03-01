use indoc::indoc;
use std::path::Path;
use test_utils::runner::{Contract, assert_passed};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
fn get_class_hash_cheatcode() {
    let test = test_case!(
        indoc!(
            r#"
            use core::clone::Clone;
            use array::ArrayTrait;
            use result::ResultTrait;
            use snforge_std::{ declare, ContractClassTrait, get_class_hash, DeclareResultTrait };

            #[test]
            fn get_class_hash_cheatcode() {
                let contract = declare("HelloStarknet").unwrap().contract_class().clone();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                assert(get_class_hash(contract_address) == contract.class_hash, 'Incorrect class hash');
            }
        "#
        ),
        Contract::from_code_path(
            "HelloStarknet".to_string(),
            Path::new("tests/data/contracts/hello_starknet.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}
