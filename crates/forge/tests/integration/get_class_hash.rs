use indoc::indoc;
use std::path::Path;
use utils::runner::Contract;
use utils::running_tests::run_test_case;
use utils::{assert_passed, test_case};

#[test]
fn get_class_hash() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use snforge_std::{ declare, ContractClassTrait, get_class_hash };

            #[test]
            fn test_get_class_hash() {
                let contract = declare('HelloStarknet');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
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

    assert_passed!(result);
}
