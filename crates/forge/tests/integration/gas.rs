use indoc::indoc;
use std::path::Path;
use test_utils::runner::Contract;
use test_utils::running_tests::run_test_case;
use test_utils::{assert_gas, assert_passed, test_case};

// gas values comes from https://book.starknet.io/ch03-01-02-fee-mechanism.html#computation
#[test]
fn declare_cost_is_omitted() {
    let test = test_case!(
        indoc!(
            r"
            use snforge_std::declare;

            #[test]
            fn declare_cost_is_omitted() {
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
    // 1 because of steps required to run the code
    assert_gas!(result, "declare_cost_is_omitted", 1);
}

#[test]
fn deploy_syscall_cost() {
    let test = test_case!(
        indoc!(
            r"
            use snforge_std::declare;
            use starknet::{SyscallResult, deploy_syscall};

            #[test]
            fn deploy_syscall_cost() {
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
    // 1224 = 2 * cost per 32-byte word (contract_address and contract modification info)
    // 612 = updated class_hash (through deploy)
    // 6 = gas cost from steps
    assert_gas!(result, "deploy_syscall_cost", 1224 + 612 + 6);
}

#[test]
fn keccak_cost() {
    let test = test_case!(indoc!(
        r"
            #[test]
            fn keccak_cost() {
                keccak::keccak_u256s_le_inputs(array![1].span());
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
    assert_gas!(result, "keccak_cost", 11);
}

#[test]
fn contract_keccak_cost() {
    let test = test_case!(
        indoc!(
            r"
            use snforge_std::{ declare, ContractClassTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn keccak(self: @TContractState);
            }

            #[test]
            fn contract_keccak_cost() {
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
    // 11 = cost of single keccak builtin
    assert_gas!(result, "contract_keccak_cost", 1836 + 11);
}

#[test]
fn range_check_cost() {
    let test = test_case!(indoc!(
        r"
            #[test]
            fn range_check_cost() {
                assert(1_u8 >= 1_u8, 'error message');
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
    assert_gas!(result, "range_check_cost", 1);
}

/// Declare, deploy and function call consume 13 `range_check_builtin`s
/// `range_check` function consumes 9, so
/// overall cost will be 22 * range check builtin cost.
/// We have to use 9 of them in the `range_check` to exceed steps cost
#[test]
fn contract_range_check_cost() {
    let test = test_case!(
        indoc!(
            r"
            use snforge_std::{ declare, ContractClassTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn range_check(self: @TContractState);
            }

            #[test]
            fn contract_range_check_cost() {
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
    // 2 = cost of 22 range check builtins
    assert_gas!(result, "contract_range_check_cost", 1836 + 2);
}

#[test]
fn bitwise_cost() {
    let test = test_case!(indoc!(
        r"
            #[test]
            fn bitwise_cost() {
                let _bitwise = 1_u8 & 1_u8;
                assert(1 == 1, 'error message');
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
    assert_gas!(result, "bitwise_cost", 1);
}

/// We have to use 6 bitwise operations in the `bitwise` function to exceed steps cost
#[test]
fn contract_bitwise_cost() {
    let test = test_case!(
        indoc!(
            r"
            use snforge_std::{ declare, ContractClassTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn bitwise(self: @TContractState);
            }

            #[test]
            fn contract_bitwise_cost() {
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
    // 2 = cost of 6 bitwise builtins
    assert_gas!(result, "contract_bitwise_cost", 1836 + 2);
}

#[test]
fn pedersen_cost() {
    let test = test_case!(indoc!(
        r"
            #[test]
            fn pedersen_cost() {
                core::pedersen::pedersen(1, 2);
                assert(1 == 1, 'error message');
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
    assert_gas!(result, "pedersen_cost", 1);
}

/// We have to use 12 pedersen operations in the `pedersen` function to exceed steps cost
#[test]
fn contract_pedersen_cost() {
    let test = test_case!(
        indoc!(
            r"
            use snforge_std::{ declare, ContractClassTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn pedersen(self: @TContractState);
            }

            #[test]
            fn contract_pedersen_cost() {
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
    // 2 = cost of 12 pedersen builtins
    assert_gas!(result, "contract_pedersen_cost", 1836 + 2);
}

#[test]
fn poseidon_cost() {
    let test = test_case!(indoc!(
        r"
            #[test]
            fn poseidon_cost() {
                core::poseidon::hades_permutation(0, 0, 0);
                assert(1 == 1, 'error message');
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
    assert_gas!(result, "poseidon_cost", 1);
}

/// We have to use 12 poseidon operations in the `poseidon` function to exceed steps cost
#[test]
fn contract_poseidon_cost() {
    let test = test_case!(
        indoc!(
            r"
            use snforge_std::{ declare, ContractClassTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn poseidon(self: @TContractState);
            }

            #[test]
            fn contract_poseidon_cost() {
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
    // 2 = cost of 12 poseidon builtins
    assert_gas!(result, "contract_poseidon_cost", 1836 + 2);
}

#[test]
fn ec_op_cost() {
    let test = test_case!(indoc!(
        r"
            use core::{ec, ec::{EcPoint, EcPointTrait}};

            #[test]
            fn ec_op_cost() {
                EcPointTrait::new_from_x(1).unwrap().mul(2);
                assert(1 == 1, 'error message');
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
    assert_gas!(result, "ec_op_cost", 6);
}

#[test]
fn contract_ec_op_cost() {
    let test = test_case!(
        indoc!(
            r"
            use snforge_std::{ declare, ContractClassTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn ec_op(self: @TContractState);
            }

            #[test]
            fn contract_ec_op_cost() {
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
    // 6 = cost of single ec_op builtin
    assert_gas!(result, "contract_ec_op_cost", 1836 + 6);
}

#[test]
fn storage_write_cost() {
    let test = test_case!(
        indoc!(
            r"
            use snforge_std::{ declare, ContractClassTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn change_balance(ref self: TContractState, new_balance: u64);
            }

            #[test]
            fn storage_write_cost() {
                let contract = declare('GasChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IGasCheckerDispatcher { contract_address };

                dispatcher.change_balance(1);
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
    // 1224 = 2 * cost per 32-byte word (storage write)
    // 3 = gas cost of steps
    assert_gas!(result, "storage_write_cost", 1836 + 1224 + 3);
}

#[test]
fn storage_write_from_test_cost() {
    let test = test_case!(indoc!(
        r"
        #[starknet::contract]
        mod Contract {
            #[storage]
            struct Storage {
                balance: felt252,
            }
        }

        use tests::test_case::Contract::balanceContractMemberStateTrait;

        #[test]
        fn storage_write_from_test_cost() {
            let mut state = Contract::contract_state_for_testing();
            state.balance.write(10);
        }
    "
    ),);

    let result = run_test_case(&test);

    assert_passed!(result);
    // 1224 = 2 * cost per 32-byte word (modified contract)
    // 1224 = 2 * cost per 32-byte word (storage write)
    // 1 = gas cost of steps
    assert_gas!(result, "storage_write_from_test_cost", 1224 + 1224 + 1);
}

#[test]
fn multiple_storage_writes_cost() {
    let test = test_case!(
        indoc!(
            r"
            use snforge_std::{ declare, ContractClassTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn change_balance(ref self: TContractState, new_balance: u64);
            }

            #[test]
            fn multiple_storage_writes_cost() {
                let contract = declare('GasChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IGasCheckerDispatcher { contract_address };

                dispatcher.change_balance(1);
                dispatcher.change_balance(1);
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
    // 1224 = 2 * cost per 32-byte word (storage write)
    // 3 = gas cost of steps
    assert_gas!(result, "multiple_storage_writes_cost", 1836 + 1224 + 3);
}

#[test]
fn l1_message_cost() {
    let test = test_case!(
        indoc!(
            r"
            use snforge_std::{ declare, ContractClassTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn send_l1_message(self: @TContractState);
            }

            #[test]
            fn l1_message_cost() {
                let contract = declare('GasChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IGasCheckerDispatcher { contract_address };

                dispatcher.send_l1_message();
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
    // 2448 = 4 * cost per 32-byte word (l2_l1_message, header is of length 3 and payload size is 1)
    // 3 = gas cost of steps
    assert_gas!(result, "l1_message_cost", 1836 + 2448 + 3);
}

#[test]
fn l1_message_from_test_cost() {
    let test = test_case!(indoc!(
        r"
        #[test]
        fn l1_message_from_test_cost() {
            starknet::send_message_to_l1_syscall(1, array![1].span()).unwrap();
        }
    "
    ),);

    let result = run_test_case(&test);

    assert_passed!(result);
    // 2448 = 4 * cost per 32-byte word (l2_l1_message, header is of length 3 and payload size is 1)
    // 1 = gas cost of steps
    assert_gas!(result, "l1_message_from_test_cost", 2448 + 1);
}

#[test]
fn l1_message_cost_for_proxy() {
    let test = test_case!(
        indoc!(
            r"
            use starknet::ContractAddress;
            use snforge_std::{ declare, ContractClassTrait };

            #[starknet::interface]
            trait IGasCheckerProxy<TContractState> {
                fn send_l1_message_from_gas_checker(
                    self: @TContractState,
                    address: ContractAddress
                );
            }

            #[test]
            fn l1_message_cost_for_proxy() {
                let contract = declare('GasChecker');
                let gas_checker_address = contract.deploy(@ArrayTrait::new()).unwrap();

                let contract = declare('GasCheckerProxy');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IGasCheckerProxyDispatcher { contract_address };

                dispatcher.send_l1_message_from_gas_checker(gas_checker_address);
            }
        "
        ),
        Contract::from_code_path(
            "GasChecker".to_string(),
            Path::new("tests/data/contracts/gas_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "GasCheckerProxy".to_string(),
            Path::new("tests/data/contracts/gas_checker_proxy.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
    // 1836 = 3 * cost per 32-byte word (deploy)
    // 1836 = 3 * cost per 32-byte word (deploy)
    // 2448 = 4 * cost per 32-byte word (l2_l1_message, header is of length 3 and payload size is 1)
    // 8 = gas cost of steps
    assert_gas!(result, "l1_message_cost_for_proxy", 1836 + 1836 + 2448 + 8);
}
