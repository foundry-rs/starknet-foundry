use forge_runner::forge_config::ForgeTrackedResource;
use indoc::indoc;
use std::path::Path;
use test_utils::runner::{Contract, assert_passed};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
fn override_entrypoint() {
    let test = test_case!(
        indoc!(
            r#"
            use core::clone::Clone;
            use snforge_std::{declare, replace_bytecode, ContractClassTrait, DeclareResultTrait};

            #[starknet::interface]
            trait IReplaceBytecode<TContractState> {
                fn get(self: @TContractState) -> felt252;
            }

            #[test]
            fn override_entrypoint() {
                let contract = declare("ReplaceBytecodeA").unwrap().contract_class();
                let contract_b_class = declare("ReplaceBytecodeB").unwrap().contract_class().class_hash.clone();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IReplaceBytecodeDispatcher { contract_address };

                assert(dispatcher.get() == 2137, '');

                replace_bytecode(contract_address, contract_b_class);

                assert(dispatcher.get() == 420, '');
            }
        "#
        ),
        Contract::from_code_path(
            "ReplaceBytecodeA",
            Path::new("tests/data/contracts/two_implementations.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "ReplaceBytecodeB",
            Path::new("tests/data/contracts/two_implementations.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn libcall_in_cheated() {
    let test = test_case!(
        indoc!(
            r#"
            use core::clone::Clone;
            use snforge_std::{declare, replace_bytecode, ContractClassTrait, DeclareResultTrait};

            #[starknet::interface]
            trait IReplaceBytecode<TContractState> {
                fn libcall(self: @TContractState, class_hash: starknet::ClassHash) -> felt252;
            }

            #[starknet::interface]
            trait ILib<TContractState> {
                fn get(self: @TContractState) -> felt252;
            }

            #[test]
            fn override_entrypoint() {
                let contract = declare("ReplaceBytecodeA").unwrap().contract_class();
                let contract_b_class = declare("ReplaceBytecodeB").unwrap().contract_class().clone().class_hash;
                let lib = declare("Lib").unwrap().contract_class().clone().class_hash;
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IReplaceBytecodeDispatcher { contract_address };

                assert(dispatcher.libcall(lib) == 123456789, '');

                replace_bytecode(contract_address, contract_b_class);

                assert(dispatcher.libcall(lib) == 123456789, '');
            }
        "#
        ),
        Contract::from_code_path(
            "Lib",
            Path::new("tests/data/contracts/two_implementations.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "ReplaceBytecodeA",
            Path::new("tests/data/contracts/two_implementations.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "ReplaceBytecodeB",
            Path::new("tests/data/contracts/two_implementations.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn contract_not_deployed() {
    let test = test_case!(
        indoc!(
            r#"
            use core::clone::Clone;
            use snforge_std::{declare, replace_bytecode, ReplaceBytecodeError, DeclareResultTrait};
            use starknet::{ClassHash, contract_address_const};

            #[test]
            fn contract_not_deployed() {
                let class_hash = declare("ReplaceBytecodeA").unwrap().contract_class().clone().class_hash;

                let non_existing_contract_address = contract_address_const::<0x2>();
                match replace_bytecode(non_existing_contract_address, class_hash) {
                    Result::Ok(()) => {
                        panic!("Wrong return type");
                    },
                    Result::Err(err) => {
                        assert(err == ReplaceBytecodeError::ContractNotDeployed(()), 'Wrong error type');
                    }
                }
            }
        "#
        ),
        Contract::from_code_path(
            "ReplaceBytecodeA",
            Path::new("tests/data/contracts/two_implementations.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn class_hash_not_declared() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{declare, ContractClassTrait, replace_bytecode, ReplaceBytecodeError, DeclareResultTrait};
            use starknet::{ClassHash, contract_address_const};

            #[test]
            fn class_hash_not_declared() {
                let contract = declare("ReplaceBytecodeA").unwrap().contract_class();
                let undeclared_class_hash: ClassHash = 0x5.try_into().unwrap();
                let (contract_address, _) = contract.deploy(@array![]).unwrap();

                match replace_bytecode(contract_address, undeclared_class_hash) {
                    Result::Ok(()) => {
                        panic!("Wrong return type");
                    },
                    Result::Err(err) => {
                        assert(err == ReplaceBytecodeError::UndeclaredClassHash(()), 'Wrong error type');
                    }
                }
            }
        "#
        ),
        Contract::from_code_path(
            "ReplaceBytecodeA",
            Path::new("tests/data/contracts/two_implementations.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}
