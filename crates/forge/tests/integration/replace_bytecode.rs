use indoc::indoc;
use std::path::Path;
use test_utils::runner::Contract;
use test_utils::running_tests::run_test_case;
use test_utils::{assert_passed, test_case};

#[test]
fn override_entrypoint() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{declare, replace_bytecode, ContractClassTrait};

            #[starknet::interface]
            trait IReplaceBytecode<TContractState> {
                fn get(self: @TContractState) -> felt252;
            }

            #[test]
            fn override_entrypoint() {
                let contract = declare("ReplaceBytecodeA");
                let contract_b_class = declare("ReplaceBytecodeB").class_hash;
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IReplaceBytecodeDispatcher { contract_address };

                assert(dispatcher.get() == 2137, '');

                replace_bytecode(contract_address, contract_b_class);

                assert(dispatcher.get() == 420, '');
            }
        "#
        ),
        Contract::from_code_path(
            "ReplaceBytecodeA".to_string(),
            Path::new("tests/data/contracts/two_implementations.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "ReplaceBytecodeB".to_string(),
            Path::new("tests/data/contracts/two_implementations.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}
