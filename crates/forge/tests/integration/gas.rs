use forge_runner::forge_config::ForgeTrackedResource;
use indoc::indoc;
use starknet_api::execution_resources::{GasAmount, GasVector};
use std::path::Path;
use test_utils::runner::{Contract, assert_gas, assert_passed};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

// all calculations are based on formulas from
// https://docs.starknet.io/architecture-and-concepts/network-architecture/fee-mechanism/#overall_fee
// important info from this link regarding gas calculations:
// 1 cairo step = 0.0025 L1 gas = 100 L2 gas
// 1 sierra gas = 1 l2 gas
// Costs of syscalls (if provided) are taken from versioned_constants (blockifier)

#[test]
fn declare_cost_is_omitted_cairo_steps() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::declare;

            #[test]
            fn declare_cost_is_omitted() {
                declare("GasChecker").unwrap();
            }
        "#
        ),
        Contract::from_code_path(
            "GasChecker".to_string(),
            Path::new("tests/data/contracts/gas_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
    // 1 = cost of 230 steps (because int(0.0025 * 230) = 1)
    //      -> as stated in the top comment, 1 cairo step = 0.0025 L1 gas = 100 L2 gas
    //         0.0025 * 230 = 0,575 (BUT rounding up to 1, since this is as little as possible)
    //         since 230 steps = 1 gas, to convert this to l2 gas we need to multiply by 40000 (100/0.0025)
    // 0 l1_gas + 0 l1_data_gas + 1 * (100 / 0.0025) l2 gas
    assert_gas(
        &result,
        "declare_cost_is_omitted",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(0),
            l2_gas: GasAmount(40000),
        },
    );
}

#[test]
fn deploy_syscall_cost_cairo_steps() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{declare, DeclareResultTrait};
            use starknet::syscalls::deploy_syscall;

            #[test]
            fn deploy_syscall_cost() {
                let contract = declare("GasConstructorChecker").unwrap().contract_class().clone();
                let (address, _) = deploy_syscall(contract.class_hash, 0, array![1].span(), false).unwrap();
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

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
    // l = 1 (updated contract class)
    // n = 1 (unique contracts updated - in this case it's the new contract address)
    // ( l + n * 2 ) * felt_size_in_bytes(32) = 96 (total l1 data cost)
    // 11 = cost of 2 keccak builtins from constructor (because int(5.12 * 2) = 11)
    // 0 l1_gas + 96 l1_data_gas + 11 * (100 / 0.0025) l2 gas
    assert_gas(
        &result,
        "deploy_syscall_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(96),
            l2_gas: GasAmount(440_000),
        },
    );
}

#[test]
fn snforge_std_deploy_cost_cairo_steps() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };

            #[test]
            fn deploy_cost() {
                let contract = declare("GasConstructorChecker").unwrap().contract_class();
                let (address, _) = contract.deploy(@array![1]).unwrap();
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

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
    // 96 = gas cost of onchain data (deploy cost)
    // 11 = cost of 2 keccak builtins = 11 (because int(5.12 * 2) = 11)
    // 0 l1_gas + 96 l1_data_gas + 11 * (100 / 0.0025) l2 gas
    assert_gas(
        &result,
        "deploy_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(96),
            l2_gas: GasAmount(440_000),
        },
    );
}

#[test]
fn keccak_cost_cairo_steps() {
    let test = test_case!(indoc!(
        r"
            #[test]
            fn keccak_cost() {
                keccak::keccak_u256s_le_inputs(array![1].span());
            }
        "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
    // 6 = cost of 1 keccak builtin (because int(5.12 * 1) = 6)
    // 0 l1_gas + 0 l1_data_gas + 6 * (100 / 0.0025) l2 gas
    assert_gas(
        &result,
        "keccak_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(0),
            l2_gas: GasAmount(240_000),
        },
    );
}

#[test]
fn contract_keccak_cost_cairo_steps() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn keccak(self: @TContractState, repetitions: u32);
            }

            #[test]
            fn contract_keccak_cost() {
                let contract = declare("GasChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
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

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
    // 96 = cost of deploy (see snforge_std_deploy_cost test)
    // 26 = cost of 5 keccak builtins (because int(5.12 * 5) = 26)
    // 0 l1_gas + 96 l1_data_gas + 26 * (100 / 0.0025) l2 gas
    assert_gas(
        &result,
        "contract_keccak_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(96),
            l2_gas: GasAmount(1_040_000),
        },
    );
}

#[test]
fn range_check_cost_cairo_steps() {
    let test = test_case!(indoc!(
        r"
            #[test]
            fn range_check_cost() {
                assert(1_u8 >= 1_u8, 'error message');
            }
        "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
    // 1 = cost of 1 range check builtin (because int(0.04 * 1) = 1)
    // 0 l1_gas + 0 l1_data_gas + 1 * (100 / 0.0025) l2 gas
    assert_gas(
        &result,
        "range_check_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(0),
            l2_gas: GasAmount(40000),
        },
    );
}

/// Declare, deploy and function call consume 13 `range_check_builtin`s
/// `range_check` function consumes 9, so
/// overall cost will be 22 * range check builtin cost.
#[test]
fn contract_range_check_cost_cairo_steps() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn range_check(self: @TContractState);
            }

            #[test]
            fn contract_range_check_cost() {
                let contract = declare("GasChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
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

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
    // 96 = cost of deploy (see snforge_std_deploy_cost test)
    // 9 = cost of 202 range check builtins (because int(0.04 * 202) = 9)
    // 0 l1_gas + 96 l1_data_gas + 9 * (100 / 0.0025) l2 gas
    assert_gas(
        &result,
        "contract_range_check_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(96),
            l2_gas: GasAmount(360_000),
        },
    );
}

#[test]
fn bitwise_cost_cairo_steps() {
    let test = test_case!(indoc!(
        r"
            #[test]
            fn bitwise_cost() {
                let _bitwise = 1_u8 & 1_u8;
                assert(1 == 1, 'error message');
            }
        "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
    // 1 = cost of 1 bitwise builtin, because int(0.16 * 1) = 1
    // 0 l1_gas + 0 l1_data_gas + 1 * (100 / 0.0025) l2 gas
    assert_gas(
        &result,
        "bitwise_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(0),
            l2_gas: GasAmount(40000),
        },
    );
}

/// We have to use 6 bitwise operations in the `bitwise` function to exceed steps cost
#[test]
fn contract_bitwise_cost_cairo_steps() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn bitwise(self: @TContractState, repetitions: u32);
            }

            #[test]
            fn contract_bitwise_cost() {
                let contract = declare("GasChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IGasCheckerDispatcher { contract_address };

                dispatcher.bitwise(300);
            }
        "#
        ),
        Contract::from_code_path(
            "GasChecker".to_string(),
            Path::new("tests/data/contracts/gas_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
    // 96 = cost of deploy l1 cost (see snforge_std_deploy_cost test)
    // 48 = cost of 300 bitwise builtins (because int(0.16 * 300) = 48)
    // 0 l1_gas + 96 l1_data_gas + 48 * (100 / 0.0025) l2 gas
    assert_gas(
        &result,
        "contract_bitwise_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(96),
            l2_gas: GasAmount(1_920_000),
        },
    );
}

#[test]
fn pedersen_cost_cairo_steps() {
    let test = test_case!(indoc!(
        r"
            #[test]
            fn pedersen_cost() {
                core::pedersen::pedersen(1, 2);
                assert(1 == 1, 'error message');
            }
        "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
    // 1 = cost of 1 pedersen builtin (because int(0.16 * 1) = 1)
    // 0 l1_gas + 0 l1_data_gas + 1 * (100 / 0.0025) l2 gas
    assert_gas(
        &result,
        "pedersen_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(0),
            l2_gas: GasAmount(40000),
        },
    );
}

/// We have to use 12 pedersen operations in the `pedersen` function to exceed steps cost
#[test]
fn contract_pedersen_cost_cairo_steps() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn pedersen(self: @TContractState);
            }

            #[test]
            fn contract_pedersen_cost() {
                let contract = declare("GasChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
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

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
    // 96 = cost of deploy (see snforge_std_deploy_cost test)
    // 10 = cost of 125 pedersen builtins (because int(0.08 * 125) = 10)
    // 0 l1_gas + 96 l1_data_gas + 10 * (100 / 0.0025) l2 gas
    assert_gas(
        &result,
        "contract_pedersen_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(96),
            l2_gas: GasAmount(400_000),
        },
    );
}

#[test]
fn poseidon_cost_cairo_steps() {
    let test = test_case!(indoc!(
        r"
            #[test]
            fn poseidon_cost() {
                core::poseidon::hades_permutation(0, 0, 0);
                assert(1 == 1, 'error message');
            }
        "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
    // 1 = cost of 1 poseidon builtin (because int(0.08 * 1) = 1)
    // 0 l1_gas + 0 l1_data_gas + 1 * (100 / 0.0025) l2 gas
    assert_gas(
        &result,
        "poseidon_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(0),
            l2_gas: GasAmount(40000),
        },
    );
}

/// We have to use 12 poseidon operations in the `poseidon` function to exceed steps cost
#[test]
fn contract_poseidon_cost_cairo_steps() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn poseidon(self: @TContractState);
            }

            #[test]
            fn contract_poseidon_cost() {
                let contract = declare("GasChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
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

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
    // 96 = cost of deploy (see snforge_std_deploy_cost test)
    // 13 = cost of 160 poseidon builtins (because int(0.08 * 160) = 13)
    // 0 l1_gas + 96 l1_data_gas + 13 * (100 / 0.0025) l2 gas
    assert_gas(
        &result,
        "contract_poseidon_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(96),
            l2_gas: GasAmount(520_000),
        },
    );
}

#[test]
fn ec_op_cost_cairo_steps() {
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

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
    // 3 = cost of 1 ec_op builtin (because int(2.56 * 1) = 3)
    // 0 l1_gas + 0 l1_data_gas + 3 * (100 / 0.0025) l2 gas
    assert_gas(
        &result,
        "ec_op_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(0),
            l2_gas: GasAmount(120_000),
        },
    );
}

#[test]
fn contract_ec_op_cost_cairo_steps() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn ec_op(self: @TContractState, repetitions: u32);
            }

            #[test]
            fn contract_ec_op_cost() {
                let contract = declare("GasChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
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

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
    // 96 = cost of deploy (see snforge_std_deploy_cost test)
    // 26 = cost of 10 ec_op builtins (because int(2.56 * 10) = 26)
    // 0 l1_gas + 96 l1_data_gas + 26 * (100 / 0.0025) l2 gas
    assert_gas(
        &result,
        "contract_ec_op_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(96),
            l2_gas: GasAmount(1_040_000),
        },
    );
}

#[test]
fn storage_write_cost_cairo_steps() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn change_balance(ref self: TContractState, new_balance: u64);
            }

            #[test]
            fn storage_write_cost() {
                let contract = declare("GasChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
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

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
    // 2576 * 0.0025 = 6.44 ~ 7 = gas cost of steps
    // 96 = gas cost of deployment
    // storage_updates(1) * 2 * 32 = 64
    // storage updates from zero value(1) * 32 = 32 (https://community.starknet.io/t/starknet-v0-13-4-pre-release-notes/115257#p-2358763-da-costs-27)
    // 0 l1_gas + (96 + 64 + 32) l1_data_gas + 7 * (100 / 0.0025) l2 gas
    assert_gas(
        &result,
        "storage_write_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(192),
            l2_gas: GasAmount(280_000),
        },
    );
}

#[test]
fn storage_write_from_test_cost_cairo_steps() {
    let test = test_case!(indoc!(
        r"
        #[starknet::contract]
        mod Contract {
            #[storage]
            struct Storage {
                balance: felt252,
            }
        }


        #[test]
        fn storage_write_from_test_cost() {
            let mut state = Contract::contract_state_for_testing();
            state.balance.write(10);
        }
    "
    ),);

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
    // 173 * 0.0025 = 0.4325 ~ 1 = gas cost of steps
    // n = unique contracts updated
    // m = values updated
    // So, as per formula:
    // n(1) * 2 * 32 = 64
    // m(1) * 2 * 32 = 64
    // storage updates from zero value(1) * 32 = 32 (https://community.starknet.io/t/starknet-v0-13-4-pre-release-notes/115257#p-2358763-da-costs-27)
    // 0 l1_gas + (64 + 64 + 32) l1_data_gas + 1 * (100 / 0.0025) l2 gas
    assert_gas(
        &result,
        "storage_write_from_test_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(160),
            l2_gas: GasAmount(40000),
        },
    );
}

#[test]
fn multiple_storage_writes_cost_cairo_steps() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn change_balance(ref self: TContractState, new_balance: u64);
            }

            #[test]
            fn multiple_storage_writes_cost() {
                let contract = declare("GasChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
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

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
    // (3763 + 8 memory holes) * 0.0025 = 9,4275 ~ 10 = gas cost of steps
    // l = number of class hash updates
    // n = unique contracts updated
    // m = unique(!) values updated
    // So, as per formula:
    // n(1) * 2 * 32 = 64
    // m(1) * 2 * 32 = 64
    // l(1) * 32 = 32
    // storage updates from zero value(1) * 32 = 32 (https://community.starknet.io/t/starknet-v0-13-4-pre-release-notes/115257#p-2358763-da-costs-27)
    // 0 l1_gas + (64 + 64 + 32 + 32) l1_data_gas + 10 * (100 / 0.0025) l2 gas
    assert_gas(
        &result,
        "multiple_storage_writes_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(192),
            l2_gas: GasAmount(400_000),
        },
    );
}

#[test]
fn l1_message_cost_cairo_steps() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn send_l1_message(self: @TContractState);
            }

            #[test]
            fn l1_message_cost() {
                let contract = declare("GasChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
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

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
    // 2614 * 0.0025 = 6.535 ~ 7 = gas cost of steps
    // 96 = gas cost of deployment
    // 29524 = gas cost of onchain data
    // 29524 l1_gas + 96 l1_data_gas + 7 * (100 / 0.0025) l2 gas
    assert_gas(
        &result,
        "l1_message_cost",
        GasVector {
            l1_gas: GasAmount(29524),
            l1_data_gas: GasAmount(96),
            l2_gas: GasAmount(280_000),
        },
    );
}

#[test]
fn l1_message_from_test_cost_cairo_steps() {
    let test = test_case!(indoc!(
        r"
        #[test]
        fn l1_message_from_test_cost() {
            starknet::send_message_to_l1_syscall(1, array![1].span()).unwrap();
        }
    "
    ),);

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
    // 224 * 0.0025 = 0.56 ~ 1 = gas cost of steps
    // 26764 = gas cost of onchain data
    // 26764 l1_gas + 0 l1_data_gas + 1 * (100 / 0.0025) l2 gas
    assert_gas(
        &result,
        "l1_message_from_test_cost",
        GasVector {
            l1_gas: GasAmount(26764),
            l1_data_gas: GasAmount(0),
            l2_gas: GasAmount(40000),
        },
    );
}

#[test]
fn l1_message_cost_for_proxy_cairo_steps() {
    let test = test_case!(
        indoc!(
            r#"
            use starknet::ContractAddress;
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };

            #[starknet::interface]
            trait IGasCheckerProxy<TContractState> {
                fn send_l1_message_from_gas_checker(
                    self: @TContractState,
                    address: ContractAddress
                );
            }

            #[test]
            fn l1_message_cost_for_proxy() {
                let contract = declare("GasChecker").unwrap().contract_class();
                let (gas_checker_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();

                let contract = declare("GasCheckerProxy").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
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

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
    // 5416 * 0.0025 = 13.54 ~ 14 = gas cost of steps
    // l = number of class hash updates
    // n = unique contracts updated
    // So, as per formula:
    // n(2) * 2 * 32 = 128
    // l(2) * 32 = 64
    // 29524 = gas cost of message
    // 29524 l1_gas + (128 + 64) l1_data_gas + 14 * (100 / 0.0025) l2 gas
    assert_gas(
        &result,
        "l1_message_cost_for_proxy",
        GasVector {
            l1_gas: GasAmount(29524),
            l1_data_gas: GasAmount(192),
            l2_gas: GasAmount(560_000),
        },
    );
}

#[test]
fn l1_handler_cost_cairo_steps() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait, L1HandlerTrait };

            #[test]
            fn l1_handler_cost() {
                let contract = declare("GasChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@array![]).unwrap();

                let mut l1_handler = L1HandlerTrait::new(contract_address, selector!("handle_l1_message"));

                l1_handler.execute(123, array![].span()).unwrap();
            }
        "#
        ),
        Contract::from_code_path(
            "GasChecker".to_string(),
            Path::new("tests/data/contracts/gas_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);
    assert_passed(&result);
    // TODO(#2960): These calculations are based on code from blockifier 0.14.0-rc.1
    // ATM, they are a bit different from the formula in the docs
    // We should verify correctness of this (docs may be outdated)

    // 96 = gas cost of onchain data (deploy cost)
    // int(5.12 * 4) = 21 = keccak cost from l1 handler
    // in this test, l1_handler_payload_size = 6
    // 15923 = 12251 (gas used for processing L1<>L2 messages on L1) + 3672 (SHARP gas, 6 * 612)
    // 12251 = 3072 (6 * 512, 512 is gas per memory word) +
    //         + 4179 (result of get_consumed_message_to_l2_emissions_cost(6) which is get_event_emission_cost(3, 3 + 6) = 375 + (3 + 1) * 375 + 9 * 256) +
    //         + 0 +
    //         + 5000 (1 * 5000, 5000 is gas per counter decrease, ref: https://github.com/starkware-libs/sequencer/blob/main/crates/blockifier/src/fee/resources.rs#L364-L368)
    //
    //
    // 15923 l1_gas + 96 l1_data_gas + 21 * (100 / 0.0025) l2 gas
    assert_gas(
        &result,
        "l1_handler_cost",
        GasVector {
            l1_gas: GasAmount(15923),
            l1_data_gas: GasAmount(96),
            l2_gas: GasAmount(840_000),
        },
    );
}

#[test]
fn events_cost_cairo_steps() {
    let test = test_case!(indoc!(
        r"
            use starknet::syscalls::emit_event_syscall;
            #[test]
            fn events_cost() {
                let mut keys = array![];
                let mut values =  array![];

                let mut i: u32 = 0;
                while i < 50 {
                    keys.append('key');
                    values.append(1);
                    i += 1;
                };

                emit_event_syscall(keys.span(), values.span()).unwrap();
            }
        "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
    // todo(3078): verify gas required be event keys and data
    // 156 range_check_builtin ~= 7
    // 6 gas for 50 event values
    // ~13 gas for 50 event keys
    // 0 l1_gas + 0 l1_data_gas + (7 + 6 + ~13) * (100 / 0.0025) l2 gas
    assert_gas(
        &result,
        "events_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(0),
            l2_gas: GasAmount(1_048_000),
        },
    );
}

#[test]
fn events_contract_cost_cairo_steps() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn emit_event(ref self: TContractState, n_keys_and_vals: u32);
            }

            #[test]
            fn event_emission_cost() {
                let (contract_address, _) = declare("GasChecker").unwrap().contract_class().deploy(@array![]).unwrap();
                let dispatcher = IGasCheckerDispatcher { contract_address };

                dispatcher.emit_event(50);
            }
        "#
        ),
        Contract::from_code_path(
            "GasChecker",
            Path::new("tests/data/contracts/gas_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);
    assert_passed(&result);
    // 4119 * 0.0025 = 10.2975 ~ 11 = gas cost of steps
    // 96 = gas cost of onchain data (deploy cost)
    // 6 gas for 50 event values
    // ~13 gas for 50 event keys
    // 0 l1_gas + 96 l1_data_gas + (11 + 6 + ~13) * (100 / 0.0025) l2 gas
    assert_gas(
        &result,
        "event_emission_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(96),
            l2_gas: GasAmount(1_208_000),
        },
    );
}

#[test]
#[cfg_attr(not(feature = "scarb_since_2_10"), ignore)]
fn declare_cost_is_omitted_sierra_gas() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::declare;
            #[test]
            fn declare_cost_is_omitted() {
                declare("GasChecker").unwrap();
            }
        "#
        ),
        Contract::from_code_path(
            "GasChecker".to_string(),
            Path::new("tests/data/contracts/gas_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::SierraGas);

    assert_passed(&result);
    // 25800 = reported consumed sierra gas
    // 0 l1_gas + 0 l1_data_gas + 25800 l2 gas
    assert_gas(
        &result,
        "declare_cost_is_omitted",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(0),
            l2_gas: GasAmount(25800),
        },
    );
}

#[test]
#[cfg_attr(not(feature = "scarb_since_2_10"), ignore)]
fn deploy_syscall_cost_sierra_gas() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{declare, DeclareResultTrait};
            use starknet::syscalls::deploy_syscall;
            #[test]
            fn deploy_syscall_cost() {
                let contract = declare("GasConstructorChecker").unwrap().contract_class().clone();
                let (address, _) = deploy_syscall(contract.class_hash, 0, array![1].span(), false).unwrap();
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

    let result = run_test_case(&test, ForgeTrackedResource::SierraGas);

    assert_passed(&result);
    // l = 1 (updated contract class)
    // n = 1 (unique contracts updated - in this case it's the new contract address)
    // ( l + n * 2 ) * felt_size_in_bytes(32) = 96 (total l1 data cost)
    //
    // 20000 = cost of 2 keccak syscall (because 2 * 100 * 100) (from constructor)
    //      -> 1 keccak syscall costs 100 cairo steps
    // 147660 = cost of 1 deploy syscall (because 1 * (1132 + 8) * 100 + (7 + 1) * 4050 + (18 + 1) * 70)
    //      -> 1 deploy syscall costs 1132 cairo steps, 7 pedersen and 18 range check builtins
    //      -> 1 calldata element costs 8 cairo steps and 1 pedersen
    //      -> 1 pedersen costs 4050, 1 range check costs 70
    // 476314 = reported consumed sierra gas
    // 0 l1_gas + 96 l1_data_gas + (20000 + 147660 + 476314) l2 gas
    assert_gas(
        &result,
        "deploy_syscall_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(96),
            l2_gas: GasAmount(643_974),
        },
    );
}

#[test]
#[cfg_attr(not(feature = "scarb_since_2_10"), ignore)]
fn snforge_std_deploy_cost_sierra_gas() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };
            #[test]
            fn deploy_cost() {
                let contract = declare("GasConstructorChecker").unwrap().contract_class();
                let (address, _) = contract.deploy(@array![1]).unwrap();
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

    let result = run_test_case(&test, ForgeTrackedResource::SierraGas);

    assert_passed(&result);
    // 96 = gas cost of onchain data (see `deploy_syscall_cost_sierra_gas` test)
    // 20000 = cost of 2 keccak syscall (see `deploy_syscall_cost_sierra_gas` test)
    // 147660 = cost of 1 deploy syscall (see `deploy_syscall_cost_sierra_gas` test)
    // 496254 = reported consumed sierra gas
    // 0 l1_gas + 96 l1_data_gas + (20000 + 147660 + 496254) l2 gas
    assert_gas(
        &result,
        "deploy_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(96),
            l2_gas: GasAmount(663_914),
        },
    );
}

#[test]
#[cfg_attr(not(feature = "scarb_since_2_10"), ignore)]
fn keccak_cost_sierra_gas() {
    let test = test_case!(indoc!(
        r"
            #[test]
            fn keccak_cost() {
                keccak::keccak_u256s_le_inputs(array![1].span());
            }
        "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::SierraGas);

    assert_passed(&result);
    // 10000 = cost of 1 keccak syscall (1 * 100 * 100)
    //      -> 1 keccak syscall costs 100 cairo steps
    // 58550 = reported consumed sierra gas
    // 0 l1_gas + 0 l1_data_gas + (10000 + 58550) l2 gas
    assert_gas(
        &result,
        "keccak_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(0),
            l2_gas: GasAmount(68550),
        },
    );
}

#[test]
#[cfg_attr(not(feature = "scarb_since_2_10"), ignore)]
fn contract_keccak_cost_sierra_gas() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };
            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn keccak(self: @TContractState, repetitions: u32);
            }
            #[test]
            fn contract_keccak_cost() {
                let contract = declare("GasChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
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

    let result = run_test_case(&test, ForgeTrackedResource::SierraGas);

    assert_passed(&result);
    // 96 = gas cost of onchain data (see `deploy_syscall_cost_sierra_gas` test)
    // 142810 = cost of 1 deploy syscall (see `deploy_syscall_cost_sierra_gas` test)
    // 50000 = cost of 5 keccak syscall (5 * 100 * 100)
    //      -> 1 keccak syscall costs 100 cairo steps
    // 87650 = cost of 1 call contract syscall (because 1 * 866 * 100 + 15 * 70)
    //      -> 1 call contract syscall costs 866 cairo steps and 15 range check builtins
    //      -> 1 range check costs 70
    // 1169195 = reported consumed sierra gas
    // 0 l1_gas + 96 l1_data_gas + (142810 + 50000 + 87650 + 1169195) l2 gas
    assert_gas(
        &result,
        "contract_keccak_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(96),
            l2_gas: GasAmount(1_449_655),
        },
    );
}

#[test]
#[cfg_attr(not(feature = "scarb_since_2_10"), ignore)]
fn contract_range_check_cost_sierra_gas() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };
            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn range_check(self: @TContractState);
            }
            #[test]
            fn contract_range_check_cost() {
                let contract = declare("GasChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
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

    let result = run_test_case(&test, ForgeTrackedResource::SierraGas);

    assert_passed(&result);
    // 96 = gas cost of onchain data (see `deploy_syscall_cost_sierra_gas` test)
    // 142810 = cost of 1 deploy syscall (see `deploy_syscall_cost_sierra_gas` test)
    // 87650 = cost of 1 call contract syscall (see `contract_keccak_cost_sierra_gas` test)
    // 140134 = reported consumed sierra gas
    // 0 l1_gas + 96 l1_data_gas + (142810 + 87650 + 140134) l2 gas
    assert_gas(
        &result,
        "contract_range_check_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(96),
            l2_gas: GasAmount(370_690),
        },
    );
}

#[test]
#[cfg_attr(not(feature = "scarb_since_2_10"), ignore)]
fn storage_write_cost_sierra_gas() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };
            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn change_balance(ref self: TContractState, new_balance: u64);
            }
            #[test]
            fn storage_write_cost() {
                let contract = declare("GasChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
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

    let result = run_test_case(&test, ForgeTrackedResource::SierraGas);

    assert_passed(&result);
    // 96 = gas cost of onchain data (see `deploy_syscall_cost_sierra_gas` test)
    // 64 = storage_updates(1) * 2 * 32
    // 32 = storage updates from zero value(1) * 32 (https://community.starknet.io/t/starknet-v0-13-4-pre-release-notes/115257#p-2358763-da-costs-27)
    // 142810 = cost of 1 deploy syscall (see `deploy_syscall_cost_sierra_gas` test)
    // 87650 = cost of 1 call contract syscall (see `contract_keccak_cost_sierra_gas` test)
    // 10000 = cost of 1 storage write syscall (because 1 * 93 * 100 + 1 * 70 = 9370)
    //      -> 1 storage write syscall costs 93 cairo steps and 1 range check builtin
    //      -> 1 range check costs 70
    //      -> the minimum total cost is `syscall_base_gas_cost`, which is pre-charged by the compiler (atm it is 100 * 100)
    // 62378 = reported consumed sierra gas
    // 0 l1_gas + (96 + 64 + 32) l1_data_gas + (142810 + 87650 + 10000 + 62378) l2 gas
    assert_gas(
        &result,
        "storage_write_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(192),
            l2_gas: GasAmount(303_030),
        },
    );
}

#[test]
#[cfg_attr(not(feature = "scarb_since_2_10"), ignore)]
fn multiple_storage_writes_cost_sierra_gas() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };
            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn change_balance(ref self: TContractState, new_balance: u64);
            }
            #[test]
            fn multiple_storage_writes_cost() {
                let contract = declare("GasChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
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

    let result = run_test_case(&test, ForgeTrackedResource::SierraGas);

    assert_passed(&result);
    // 64 = n(1) * 2 * 32
    // 64 = m(1) * 2 * 32
    // 32 = l(1) * 32
    //      -> l = number of class hash updates
    //      -> n = unique contracts updated
    //      -> m = unique(!) values updated
    // 32 = storage updates from zero value(1) * 32 (https://community.starknet.io/t/starknet-v0-13-4-pre-release-notes/115257#p-2358763-da-costs-27)
    // 142810 = cost of 1 deploy syscall (see `deploy_syscall_cost_sierra_gas` test)
    // 175300 = cost of 2 call contract syscalls (because 2 * 866 * 100 + 2 * 15 * 70)
    //      -> 1 call contract syscall costs 866 cairo steps and 15 range check builtins
    //      -> 1 range check costs 70
    // 20000 = cost of 2 storage write syscall (because 2 * 93 * 100 + 2 * 70 = 18740)
    //      -> 1 storage write syscall costs 93 cairo steps and 1 range check builtin
    //      -> 1 range check costs 70
    //      -> the minimum total cost is `syscall_base_gas_cost`, which is pre-charged by the compiler (atm it is 100 * 100)
    // 74140 = reported consumed sierra gas
    // 0 l1_gas + (64 + 64 + 32 + 32) l1_data_gas + (142810 + 175300 + 20000 + 74140) l2 gas
    assert_gas(
        &result,
        "multiple_storage_writes_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(192),
            l2_gas: GasAmount(412_070),
        },
    );
}

#[test]
#[cfg_attr(not(feature = "scarb_since_2_10"), ignore)]
fn l1_message_cost_sierra_gas() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };
            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn send_l1_message(self: @TContractState);
            }
            #[test]
            fn l1_message_cost() {
                let contract = declare("GasChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
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

    let result = run_test_case(&test, ForgeTrackedResource::SierraGas);

    assert_passed(&result);
    // todo(2960): verify l2 -> l1 message cost
    // 29524 = gas cost of l2 -> l1 message
    // 96 = gas cost of onchain data (see `deploy_syscall_cost_sierra_gas` test)
    // 142810 = cost of 1 deploy syscall (see `deploy_syscall_cost_sierra_gas` test)
    // 87650 = cost of 1 call contract syscall (see `contract_keccak_cost_sierra_gas` test)
    // 14170 = cost of 1 SendMessageToL1 syscall (because 1 * 141 * 100 + 1 * 70 )
    //      -> 1 storage write syscall costs 141 cairo steps and 1 range check builtin
    //      -> 1 range check costs 70
    // 61430 = reported consumed sierra gas
    // 29524 l1_gas + 96 l1_data_gas + (142810 + 87650 + 14170 + 61430) l2 gas
    assert_gas(
        &result,
        "l1_message_cost",
        GasVector {
            l1_gas: GasAmount(29524),
            l1_data_gas: GasAmount(96),
            l2_gas: GasAmount(306_060),
        },
    );
}

#[test]
#[cfg_attr(not(feature = "scarb_since_2_10"), ignore)]
fn l1_message_cost_for_proxy_sierra_gas() {
    let test = test_case!(
        indoc!(
            r#"
            use starknet::ContractAddress;
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };
            #[starknet::interface]
            trait IGasCheckerProxy<TContractState> {
                fn send_l1_message_from_gas_checker(
                    self: @TContractState,
                    address: ContractAddress
                );
            }
            #[test]
            fn l1_message_cost_for_proxy() {
                let contract = declare("GasChecker").unwrap().contract_class();
                let (gas_checker_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let contract = declare("GasCheckerProxy").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
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

    let result = run_test_case(&test, ForgeTrackedResource::SierraGas);

    assert_passed(&result);
    // todo(2960): verify l2 -> l1 message cost
    // 29524 = gas cost of l2 -> l1 message
    // 128 = n(2) * 2 * 32
    // 64 = l(2) * 32
    //      -> l = number of class hash updates
    //      -> n = unique contracts updated
    // 285620 = cost of 2 deploy syscall (because 2 * 1132 * 100 + 2 * 7 * 4050 + 2 * 18 * 70)
    //      -> 1 deploy syscall costs 1132 cairo steps, 7 pedersen and 18 range check builtins
    //      -> 1 pedersen costs 4050, 1 range check costs 70
    // 175300 = cost of 2 call contract syscalls (see `multiple_storage_writes_cost_sierra_gas` test)
    // 14170 = cost of 1 SendMessageToL1 syscall (see `l1_message_cost_sierra_gas` test)
    // 132800 = reported consumed sierra gas
    // 29524 l1_gas + (128 + 64) l1_data_gas + (285620 + 175300 + 14170 + 132800) l2 gas
    assert_gas(
        &result,
        "l1_message_cost_for_proxy",
        GasVector {
            l1_gas: GasAmount(29524),
            l1_data_gas: GasAmount(192),
            l2_gas: GasAmount(607_890),
        },
    );
}

#[test]
#[cfg_attr(not(feature = "scarb_since_2_10"), ignore)]
fn events_cost_sierra_gas() {
    let test = test_case!(indoc!(
        r"
            use starknet::syscalls::emit_event_syscall;
            #[test]
            fn events_cost() {
                let mut keys = array![];
                let mut values =  array![];
                let mut i: u32 = 0;
                while i < 50 {
                    keys.append('key');
                    values.append(1);
                    i += 1;
                };
                emit_event_syscall(keys.span(), values.span()).unwrap();
            }
        "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::SierraGas);

    assert_passed(&result);
    // 512000 = 50 * 10240
    //      -> we emit 50 keys, each taking up 1 felt of space
    //      -> L2 gas cost for event key is 10240 gas/felt
    // 256000 = 50 * 5120
    //      -> we emit 50 keys, each having 1 felt of data
    //      -> L2 gas cost for event data is 5120 gas/felt
    // 10000 = cost of 1 emit event syscall (because 1 * 61 * 100 + 1 * 70 = 6170)
    //      -> 1 emit event syscall costs 61 cairo steps and 1 range check builtin
    //      -> 1 range check costs 70
    //      -> the minimum total cost is `syscall_base_gas_cost`, which is pre-charged by the compiler (atm it is 100 * 100)
    // 188920 = reported consumed sierra gas
    // 0 l1_gas + 0 l1_data_gas + (512000 + 256000 + 10000 + 188920) l2 gas
    assert_gas(
        &result,
        "events_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(0),
            l2_gas: GasAmount(966_920),
        },
    );
}

#[test]
#[cfg_attr(not(feature = "scarb_since_2_10"), ignore)]
fn events_contract_cost_sierra_gas() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };
            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn emit_event(ref self: TContractState, n_keys_and_vals: u32);
            }
            #[test]
            fn event_emission_cost() {
                let (contract_address, _) = declare("GasChecker").unwrap().contract_class().deploy(@array![]).unwrap();
                let dispatcher = IGasCheckerDispatcher { contract_address };
                dispatcher.emit_event(50);
            }
        "#
        ),
        Contract::from_code_path(
            "GasChecker",
            Path::new("tests/data/contracts/gas_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::SierraGas);
    assert_passed(&result);
    // 96 = gas cost of onchain data (see `deploy_syscall_cost_sierra_gas` test)
    // 512000 = event keys cost (see `events_contract_cost_sierra_gas` test)
    // 256000 = event data cost (see `events_contract_cost_sierra_gas` test)
    // 10000 = cost of 1 emit event syscall (see `events_contract_cost_sierra_gas` test)
    // 142810 = cost of 1 deploy syscall (see `deploy_syscall_cost_sierra_gas` test)
    // 87650 = cost of 1 call contract syscall (see `contract_keccak_cost_sierra_gas` test)
    // 240714 = reported consumed sierra gas
    // 0 l1_gas + 96 l1_data_gas + (512000 + 256000 + 10000 + 142810 + 87650 + 240714) l2 gas
    assert_gas(
        &result,
        "event_emission_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(96),
            l2_gas: GasAmount(1_249_270),
        },
    );
}
