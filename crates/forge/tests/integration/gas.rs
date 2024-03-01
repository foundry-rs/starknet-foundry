use indoc::indoc;
use std::path::Path;
use test_utils::runner::Contract;
use test_utils::running_tests::run_test_case;
use test_utils::{assert_gas, assert_passed, test_case};

// all calculations are based on formula from
// https://docs.starknet.io/documentation/architecture_and_concepts/Network_Architecture/fee-mechanism/#overall_fee

#[test]
fn declare_cost_is_omitted() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::declare;

            #[test]
            fn declare_cost_is_omitted() {
                declare("GasChecker");
            }
        "#
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
            r#"
            use snforge_std::declare;
            use starknet::{SyscallResult, deploy_syscall};

            #[test]
            fn deploy_syscall_cost() {
                let contract = declare("GasConstructorChecker");
                let (address, _) = deploy_syscall(contract.class_hash, 0, array![].span(), false).unwrap();
                assert(address != 0.try_into().unwrap(), 'wrong deployed addr');
            }
        "#
        ),
        Contract::from_code_path(
            "GasConstructorChecker".to_string(),
            Path::new("tests/data/contracts/gas_constructor_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
    // 1101 = gas cost of onchain data (deploy cost)
    // int(10.24 * 2) = 21 = keccak cost from constructor
    assert_gas!(result, "deploy_syscall_cost", 1101 + 21);
}

#[test]
fn snforge_std_deploy_cost() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{ declare, ContractClassTrait };
            
            #[test]
            fn deploy_cost() {
                let contract = declare("GasConstructorChecker");
                let address = contract.deploy(@array![]).unwrap();
                assert(address != 0.try_into().unwrap(), 'wrong deployed addr');
            }
        "#
        ),
        Contract::from_code_path(
            "GasConstructorChecker".to_string(),
            Path::new("tests/data/contracts/gas_constructor_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
    // 1101 = gas cost of onchain data (deploy cost)
    // int(10.24 * 2) = 21 = keccak cost from constructor
    assert_gas!(result, "deploy_cost", 1101 + 21);
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
            r#"
            use snforge_std::{ declare, ContractClassTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn keccak(self: @TContractState, repetitions: u32);
            }

            #[test]
            fn contract_keccak_cost() {
                let contract = declare("GasChecker");
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IGasCheckerDispatcher { contract_address };

                dispatcher.keccak(5);
            }
        "#
        ),
        Contract::from_code_path(
            "GasChecker".to_string(),
            Path::new("tests/data/contracts/gas_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
    // 1101 = cost of deploy (see snforge_std_deploy_cost test)
    // 52 = cost of 5x keccak builtin
    assert_gas!(result, "contract_keccak_cost", 1101 + 52);
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
#[test]
fn contract_range_check_cost() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{ declare, ContractClassTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn range_check(self: @TContractState);
            }

            #[test]
            fn contract_range_check_cost() {
                let contract = declare("GasChecker");
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IGasCheckerDispatcher { contract_address };

                dispatcher.range_check();
            }
        "#
        ),
        Contract::from_code_path(
            "GasChecker".to_string(),
            Path::new("tests/data/contracts/gas_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
    // 1101 = cost of deploy (see snforge_std_deploy_cost test)
    // 16 = cost of 192 range check builtins
    assert_gas!(result, "contract_range_check_cost", 1101 + 16);
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
            r#"
            use snforge_std::{ declare, ContractClassTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn bitwise(self: @TContractState, repetitions: u32);
            }

            #[test]
            fn contract_bitwise_cost() {
                let contract = declare("GasChecker");
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IGasCheckerDispatcher { contract_address };

                dispatcher.bitwise(200);
            }
        "#
        ),
        Contract::from_code_path(
            "GasChecker".to_string(),
            Path::new("tests/data/contracts/gas_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
    // 1101 = cost of deploy l1 cost (see snforge_std_deploy_cost test)
    // 64 = cost of 200 bitwise builtins
    assert_gas!(result, "contract_bitwise_cost", 1101 + 64);
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
            r#"
            use snforge_std::{ declare, ContractClassTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn pedersen(self: @TContractState);
            }

            #[test]
            fn contract_pedersen_cost() {
                let contract = declare("GasChecker");
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IGasCheckerDispatcher { contract_address };

                dispatcher.pedersen();
            }
        "#
        ),
        Contract::from_code_path(
            "GasChecker".to_string(),
            Path::new("tests/data/contracts/gas_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
    // 1101 = cost of deploy (see snforge_std_deploy_cost test)
    // 14 = cost of 86 pedersen builtins
    assert_gas!(result, "contract_pedersen_cost", 1101 + 14);
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
            r#"
            use snforge_std::{ declare, ContractClassTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn poseidon(self: @TContractState);
            }

            #[test]
            fn contract_poseidon_cost() {
                let contract = declare("GasChecker");
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IGasCheckerDispatcher { contract_address };

                dispatcher.poseidon();
                dispatcher.poseidon();
            }
        "#
        ),
        Contract::from_code_path(
            "GasChecker".to_string(),
            Path::new("tests/data/contracts/gas_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
    // 1101 = cost of deploy (see snforge_std_deploy_cost test)
    // 26 = cost of 160 poseidon builtins
    assert_gas!(result, "contract_poseidon_cost", 1101 + 26);
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
            r#"
            use snforge_std::{ declare, ContractClassTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn ec_op(self: @TContractState, repetitions: u32);
            }

            #[test]
            fn contract_ec_op_cost() {
                let contract = declare("GasChecker");
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IGasCheckerDispatcher { contract_address };

                dispatcher.ec_op(10);
            }
        "#
        ),
        Contract::from_code_path(
            "GasChecker".to_string(),
            Path::new("tests/data/contracts/gas_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
    // 1101 = cost of deploy (see snforge_std_deploy_cost test)
    // 52 = cost of 10x ec_op builtins
    assert_gas!(result, "contract_ec_op_cost", 1101 + 52);
}

#[test]
fn storage_write_cost() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{ declare, ContractClassTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn change_balance(ref self: TContractState, new_balance: u64);
            }

            #[test]
            fn storage_write_cost() {
                let contract = declare("GasChecker");
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IGasCheckerDispatcher { contract_address };

                dispatcher.change_balance(1);
            }
        "#
        ),
        Contract::from_code_path(
            "GasChecker".to_string(),
            Path::new("tests/data/contracts/gas_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
    // 11 = gas cost of steps
    // 2203 = gas cost of onchain data
    assert_gas!(result, "storage_write_cost", 11 + 2203);
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
    // 1 = gas cost of steps
    // 1652 = gas cost of onchain data
    assert_gas!(result, "storage_write_from_test_cost", 1 + 1652);
}

#[test]
fn multiple_storage_writes_cost() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{ declare, ContractClassTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn change_balance(ref self: TContractState, new_balance: u64);
            }

            #[test]
            fn multiple_storage_writes_cost() {
                let contract = declare("GasChecker");
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IGasCheckerDispatcher { contract_address };

                dispatcher.change_balance(1);
                dispatcher.change_balance(1);
            }
        "#
        ),
        Contract::from_code_path(
            "GasChecker".to_string(),
            Path::new("tests/data/contracts/gas_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
    // 16 = gas cost of steps
    // 2203 = gas cost of onchain data
    assert_gas!(result, "multiple_storage_writes_cost", 16 + 2203);
}

#[test]
fn l1_message_cost() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{ declare, ContractClassTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn send_l1_message(self: @TContractState);
            }

            #[test]
            fn l1_message_cost() {
                let contract = declare("GasChecker");
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IGasCheckerDispatcher { contract_address };

                dispatcher.send_l1_message();
            }
        "#
        ),
        Contract::from_code_path(
            "GasChecker".to_string(),
            Path::new("tests/data/contracts/gas_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
    // 12 = gas cost of steps
    // 1101 = gas cost of deployment
    // 29524 = gas cost of onchain data
    assert_gas!(result, "l1_message_cost", 11 + 1101 + 29524);
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
    // 1 = gas cost of steps
    // 26764 = gas cost of onchain data
    assert_gas!(result, "l1_message_from_test_cost", 1 + 26764);
}

#[test]
fn l1_message_cost_for_proxy() {
    let test = test_case!(
        indoc!(
            r#"
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
                let contract = declare("GasChecker");
                let gas_checker_address = contract.deploy(@ArrayTrait::new()).unwrap();

                let contract = declare("GasCheckerProxy");
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IGasCheckerProxyDispatcher { contract_address };

                dispatcher.send_l1_message_from_gas_checker(gas_checker_address);
            }
        "#
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
    // 22 = gas cost of steps
    // 2442 = gas cost of l1 data (deployments - discounts)
    // 29524 = gas cost of message
    assert_gas!(result, "l1_message_cost_for_proxy", 21 + 2442 + 29524);
}

#[test]
fn l1_handler_cost() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{ declare, ContractClassTrait, L1HandlerTrait };

            #[test]
            fn l1_handler_cost() {
                let contract = declare("GasChecker");
                let contract_address = contract.deploy(@array![]).unwrap();
                
                let mut l1_handler = L1HandlerTrait::new(contract_address, function_name: 'handle_l1_message');

                l1_handler.from_address = 123;
                l1_handler.payload = array![].span();
            
                l1_handler.execute().unwrap();
            }
        "#
        ),
        Contract::from_code_path(
            "GasChecker".to_string(),
            Path::new("tests/data/contracts/gas_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);
    assert_passed!(result);
    // 1101 = gas cost of onchain data (deploy cost)
    // int(10.24 * 4) = 41 = keccak cost from l1 handler
    // 14643 - l1 cost of payload + emit message handle event
    assert_gas!(result, "l1_handler_cost", 1101 + 41 + 14643);
}
