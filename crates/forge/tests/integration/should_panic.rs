use std::path::Path;

use indoc::indoc;
use test_utils::runner::Contract;
use test_utils::running_tests::run_test_case;
use test_utils::{assert_passed, test_case};

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

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn should_panic_uknown_entry_point() {
    let test = test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use starknet::{call_contract_syscall, ContractAddress, Felt252TryIntoContractAddress};
            use result::ResultTrait;

            use snforge_std::{declare, ContractClass, ContractClassTrait};

            #[test]
            #[should_panic(expected: "Entry point selector 0x00000000000000000000696e6578697374656e745f656e7472795f706f696e74 not found in contract 0x06288882237f586f11e7a1bcdd1b4841708747bec96952dce019cd76ff3d806f")]
            fn should_panic_with_no_expected_data() {
                let contract = declare('HelloStarknet');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
            
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

    let result = run_test_case(&test);

    assert_passed!(result);
}
