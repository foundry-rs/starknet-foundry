use indoc::indoc;
use std::path::Path;
use test_utils::runner::{assert_passed, Contract};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
fn get_class_hash_cheatcode() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use snforge_std::{ declare, ContractClassTrait, get_class_hash };

            #[test]
            fn get_class_hash_cheatcode() {
                let contract = declare("HelloStarknet").unwrap();
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
