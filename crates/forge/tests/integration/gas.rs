use indoc::indoc;
use std::path::Path;
use test_utils::runner::Contract;
use test_utils::running_tests::run_test_case;
use test_utils::{assert_gas, assert_passed, test_case};

// gas values comes from https://book.starknet.io/ch03-01-02-fee-mechanism.html#computation
#[test]
fn test_declare_cost_is_omitted() {
    let test = test_case!(
        indoc!(
            r"
            use snforge_std::declare;

            #[test]
            fn test_declare() {
                declare('GasChecker');
            }
        "
        ),
        Contract::from_code_path(
            "GasChecker".to_string(),
            Path::new("tests/data/contracts/gas_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
    // 1 - initial gas cost (consists of steps required to run the code)
    assert_gas!(result, "test_declare", 1);
}

#[test]
fn test_deploy_syscall_cost() {
    let test = test_case!(
        indoc!(
            r"
            use snforge_std::declare;
            use starknet::{SyscallResult, deploy_syscall};

            #[test]
            fn test_deploy_syscall() {
                let contract = declare('GasChecker');
                deploy_syscall(contract.class_hash, 0, array![].span(), false);
            }
        "
        ),
        Contract::from_code_path(
            "GasChecker".to_string(),
            Path::new("tests/data/contracts/gas_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
    // 1224 = 2 * cost per 32-byte word (contract_address and class_hash)
    // 612 - updated class (through deploy)
    // 11 - gas cost from steps
    assert_gas!(result, "test_deploy_syscall", 1224 + 612 + 11);
}

#[test]
fn test_keccak_cost() {
    let test = test_case!(indoc!(
        r"
            #[test]
            fn test_keccak_builtin() {
                keccak::keccak_u256s_le_inputs(array![1].span());
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
    assert_gas!(result, "test_keccak_builtin", 21);
}

#[test]
fn test_contract_keccak_cost() {
    let test = test_case!(
        indoc!(
            r"
            use snforge_std::{ declare, ContractClassTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn keccak(self: @TContractState);
            }

            #[test]
            fn test_keccak_builtin() {
                let contract = declare('GasChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IGasCheckerDispatcher { contract_address };

                dispatcher.keccak();
            }
        "
        ),
        Contract::from_code_path(
            "GasChecker".to_string(),
            Path::new("tests/data/contracts/gas_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
    // 1836 = 3 * cost per 32-byte word (deploy)
    // 21 - cost of single keccak builtin
    assert_gas!(result, "test_keccak_builtin", 1836 + 21);
}

#[test]
fn test_range_check_cost() {
    let test = test_case!(indoc!(
        r"
            #[test]
            fn test_range_check() {
                assert(1_u8 >= 1_u8, 'error message');
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
    assert_gas!(result, "test_range_check", 1);
}

/// Declare, deploy and function call consume 13 `range_check_builtin`s
/// `range_check` function consumes 9, so
/// overall cost will be 22 * range check builtin cost.
/// We have to use 9 of them in the `range_check` to exceed steps cost
#[test]
fn test_contract_range_check_cost() {
    let test = test_case!(
        indoc!(
            r"
            use snforge_std::{ declare, ContractClassTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn range_check(self: @TContractState);
            }

            #[test]
            fn test_range_check() {
                let contract = declare('GasChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IGasCheckerDispatcher { contract_address };

                dispatcher.range_check();
            }
        "
        ),
        Contract::from_code_path(
            "GasChecker".to_string(),
            Path::new("tests/data/contracts/gas_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
    // 1836 = 3 * cost per 32-byte word (deploy)
    // 4 - cost of 22 range check builtins
    assert_gas!(result, "test_range_check", 1836 + 4);
}

#[test]
fn test_bitwise_cost() {
    let test = test_case!(indoc!(
        r"
            #[test]
            fn test_bitwise() {
                let bitwise = 1_u8 & 1_u8;
                assert(1 == 1, 'error message');
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
    assert_gas!(result, "test_bitwise", 1);
}

/// We have to use 6 bitwise operations in the `bitwise` function to exceed steps cost
#[test]
fn test_contract_bitwise_cost() {
    let test = test_case!(
        indoc!(
            r"
            use snforge_std::{ declare, ContractClassTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn bitwise(self: @TContractState);
            }

            #[test]
            fn test_bitwise() {
                let contract = declare('GasChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IGasCheckerDispatcher { contract_address };

                dispatcher.bitwise();
            }
        "
        ),
        Contract::from_code_path(
            "GasChecker".to_string(),
            Path::new("tests/data/contracts/gas_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
    // 1836 = 3 * cost per 32-byte word (deploy)
    // 4 - cost of 6 bitwise builtins
    assert_gas!(result, "test_bitwise", 1836 + 4);
}

#[test]
fn test_pedersen_cost() {
    let test = test_case!(indoc!(
        r"
            #[test]
            fn test_pedersen() {
                core::pedersen::pedersen(1, 2);
                assert(1 == 1, 'error message');
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
    assert_gas!(result, "test_pedersen", 1);
}

/// We have to use 12 pedersen operations in the `pedersen` function to exceed steps cost
#[test]
fn test_contract_pedersen_cost() {
    let test = test_case!(
        indoc!(
            r"
            use snforge_std::{ declare, ContractClassTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn pedersen(self: @TContractState);
            }

            #[test]
            fn test_pedersen() {
                let contract = declare('GasChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IGasCheckerDispatcher { contract_address };

                dispatcher.pedersen();
            }
        "
        ),
        Contract::from_code_path(
            "GasChecker".to_string(),
            Path::new("tests/data/contracts/gas_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
    // 1836 = 3 * cost per 32-byte word (deploy)
    // 4 - cost of 12 pedersen builtins
    assert_gas!(result, "test_pedersen", 1836 + 4);
}

#[test]
fn test_poseidon_cost() {
    let test = test_case!(indoc!(
        r"
            #[test]
            fn test_poseidon() {
                core::poseidon::hades_permutation(0, 0, 0);
                assert(1 == 1, 'error message');
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
    assert_gas!(result, "test_poseidon", 1);
}

/// We have to use 12 poseidon operations in the `poseidon` function to exceed steps cost
#[test]
fn test_contract_poseidon_cost() {
    let test = test_case!(
        indoc!(
            r"
            use snforge_std::{ declare, ContractClassTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn poseidon(self: @TContractState);
            }

            #[test]
            fn test_poseidon() {
                let contract = declare('GasChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IGasCheckerDispatcher { contract_address };

                dispatcher.poseidon();
            }
        "
        ),
        Contract::from_code_path(
            "GasChecker".to_string(),
            Path::new("tests/data/contracts/gas_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
    // 1836 = 3 * cost per 32-byte word (deploy)
    // 4 - cost of 12 poseidon builtins
    assert_gas!(result, "test_poseidon", 1836 + 4);
}

#[test]
fn test_ec_op_cost() {
    let test = test_case!(indoc!(
        r"
            use core::{ec, ec::{EcPoint, EcPointTrait}};

            #[test]
            fn test_ec_op() {
                EcPointTrait::new_from_x(1).unwrap().mul(2);
                assert(1 == 1, 'error message');
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
    assert_gas!(result, "test_ec_op", 11);
}

#[test]
fn test_contract_ec_op_cost() {
    let test = test_case!(
        indoc!(
            r"
            use snforge_std::{ declare, ContractClassTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn ec_op(self: @TContractState);
            }

            #[test]
            fn test_ec_op() {
                let contract = declare('GasChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IGasCheckerDispatcher { contract_address };

                dispatcher.ec_op();
            }
        "
        ),
        Contract::from_code_path(
            "GasChecker".to_string(),
            Path::new("tests/data/contracts/gas_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
    // 1836 = 3 * cost per 32-byte word (deploy)
    // 1 - cost of single ec_op builtin
    assert_gas!(result, "test_ec_op", 1836 + 11);
}
