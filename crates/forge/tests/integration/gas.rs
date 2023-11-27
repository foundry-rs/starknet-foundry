use indoc::indoc;
use std::path::Path;
use test_utils::runner::Contract;
use test_utils::running_tests::run_test_case;
use test_utils::{assert_gas, assert_passed, test_case};

// gas values comes from https://book.starknet.io/ch03-01-02-fee-mechanism.html#computation
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
    assert_gas!(result, "test_keccak_builtin", 20.48);
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
    assert_gas!(result, "test_keccak_builtin", 20.48);
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
    assert_gas!(result, "test_range_check", 0.16);
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
    assert_gas!(result, "test_range_check", 22.0 * 0.16);
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
    assert_gas!(result, "test_bitwise", 0.64);
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
    assert_gas!(result, "test_bitwise", 6.0 * 0.64);
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
    assert_gas!(result, "test_pedersen", 0.32);
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
    assert_gas!(result, "test_pedersen", 12.0 * 0.32);
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
    assert_gas!(result, "test_poseidon", 0.32);
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
    assert_gas!(result, "test_poseidon", 12.0 * 0.32);
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
    assert_gas!(result, "test_ec_op", 10.24);
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
    assert_gas!(result, "test_ec_op", 10.24);
}
