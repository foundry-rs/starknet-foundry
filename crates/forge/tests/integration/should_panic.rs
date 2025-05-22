use forge_runner::forge_config::ForgeTrackedResource;
use foundry_ui::Ui;
use std::path::Path;

use indoc::indoc;
use test_utils::runner::{Contract, assert_passed};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
fn should_panic() {
    let test = test_case!(indoc!(
        r"
            use array::ArrayTrait;

            #[test]
            #[should_panic]
            fn should_panic_with_no_expected_data() {
                panic_with_felt252(0);
            }

            #[test]
            #[should_panic(expected: ('panic message', ))]
            fn should_panic_check_data() {
                panic_with_felt252('panic message');
            }

            #[test]
            #[should_panic(expected: ('panic message', 'second message',))]
            fn should_panic_multiple_messages(){
                let mut arr = ArrayTrait::new();
                arr.append('panic message');
                arr.append('second message');
                panic(arr);
            }
        "
    ));

    let ui = Ui::default();
    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps, &ui);

    assert_passed(&result);
}

#[test]
fn should_panic_unknown_entry_point() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use starknet::{call_contract_syscall, ContractAddress, Felt252TryIntoContractAddress};
            use result::ResultTrait;

            use snforge_std::{declare, ContractClass, ContractClassTrait, DeclareResultTrait};

            #[test]
            #[should_panic]
            fn should_panic_with_no_expected_data() {
                let contract = declare("HelloStarknet").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();

                match call_contract_syscall(
                    contract_address,
                    'inexistent_entry_point',
                    ArrayTrait::<felt252>::new().span()
                ) {
                    Result::Ok(_) => panic_with_felt252('Expected an error'),
                    Result::Err(err) => panic(err),
                };
            }
        "#
        ),
        Contract::from_code_path(
            "HelloStarknet".to_string(),
            Path::new("tests/data/contracts/hello_starknet.cairo"),
        )
        .unwrap()
    );

    let ui = Ui::default();
    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps, &ui);

    assert_passed(&result);
}
