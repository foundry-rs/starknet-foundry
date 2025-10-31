use forge_runner::forge_config::ForgeTrackedResource;
use indoc::indoc;
use std::path::Path;
use test_utils::runner::{Contract, assert_passed};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
fn test_get_current_step() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::testing::get_current_step;
            use snforge_std::{ContractClassTrait, DeclareResultTrait, declare};

            #[test]
            fn check_current_step() {
                let step_start = get_current_step();

                let contract = declare("HelloStarknet").unwrap().contract_class().clone();
                let _ = contract.deploy(@ArrayTrait::new()).unwrap();

                let step_end = get_current_step();

                assert!(step_end > step_start);
            }
        "#
        ),
        Contract::from_code_path(
            "HelloStarknet".to_string(),
            Path::new("tests/data/contracts/hello_starknet.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::SierraGas);

    assert_passed(&result);
}
