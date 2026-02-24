use crate::utils::runner::{Contract, assert_gas, assert_passed};
use crate::utils::running_tests::run_test_case;
use crate::utils::test_case;
use forge_runner::forge_config::ForgeTrackedResource;
use indoc::{formatdoc, indoc};
use scarb_api::version::scarb_version;
use semver::Version;
use shared::test_utils::node_url::node_rpc_url;
use starknet_api::execution_resources::{GasAmount, GasVector};
use std::path::Path;

// all calculations are based on formulas from
// https://docs.starknet.io/architecture-and-concepts/fees/#overall_fee
// important info from this link regarding gas calculations:
// 1 cairo step = 0.0025 L1 gas = 100 L2 gas
// 1 sierra gas = 1 l2 gas
// Costs of syscalls (if provided) are taken from versioned_constants (blockifier)
// In Sierra gas tests, only the most significant costs are considered
// Asserted values should be slightly greater than the computed values,
// but must remain within a reasonable range

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

                dispatcher.keccak(7);
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
    // 26 = cost of 5 keccak builtins (because int(5.12 * 7) = 36)
    // 0 l1_gas + 96 l1_data_gas + 36 * (100 / 0.0025) l2 gas
    assert_gas(
        &result,
        "contract_keccak_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(96),
            l2_gas: GasAmount(1_440_000),
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

    let scarb_version = scarb_version().expect("Failed to get scarb version").scarb;

    // TODO(#4087): Remove this when bumping minimal recommended Scarb version to 2.14.0
    if scarb_version >= Version::new(2, 14, 0) {
        // 96 = cost of deploy (see snforge_std_deploy_cost test)
        // 43 = cost of 1052 range check builtins (because int(0.04 * 1052) = 43)
        // 0 l1_gas + 96 l1_data_gas + 43 * (100 / 0.0025) l2 gas
        assert_gas(
            &result,
            "contract_range_check_cost",
            GasVector {
                l1_gas: GasAmount(0),
                l1_data_gas: GasAmount(96),
                l2_gas: GasAmount(1_720_000),
            },
        );
    } else {
        assert_gas(
            &result,
            "contract_range_check_cost",
            GasVector {
                l1_gas: GasAmount(0),
                l1_data_gas: GasAmount(96),
                l2_gas: GasAmount(6_520_000),
            },
        );
    }
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
    // 96 = cost of 600 bitwise builtins (because int(0.16 * 600) = 96)
    // 0 l1_gas + 96 l1_data_gas + 96 * (100 / 0.0025) l2 gas
    assert_gas(
        &result,
        "contract_bitwise_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(96),
            l2_gas: GasAmount(3_840_000),
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
            l2_gas: GasAmount(360_000),
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
    // (4960 steps + 28 memory holes) * 0.0025 = 12.47 ~ 13 = gas cost of steps
    // l = number of class hash updates
    // n = unique contracts updated
    // So, as per formula:
    // n(2) * 2 * 32 = 128
    // l(2) * 32 = 64
    // 29524 = gas cost of message
    // 29524 l1_gas + (128 + 64) l1_data_gas + 13 * (100 / 0.0025) l2 gas
    assert_gas(
        &result,
        "l1_message_cost_for_proxy",
        GasVector {
            l1_gas: GasAmount(29524),
            l1_data_gas: GasAmount(192),
            l2_gas: GasAmount(520_000),
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
    // 96 = gas cost of onchain data (deploy cost)
    // int(5.12 * 4) = 21 = keccak cost from l1 handler
    // in this test, l1_handler_payload_size = 6
    // 15923 = 12251 (gas used for processing L1<>L2 messages on L1) + 3672 (SHARP gas, 6 * 612)
    // https://github.com/starkware-libs/sequencer/blob/028db0341378147037b5e7236d8e136e4ca7c30d/crates/blockifier/src/fee/resources.rs#L338-L340
    // 12251 = 3072 (6 * 512, 512 is gas per memory word) +
    //         + 4179 (cost of `ConsumedMessageToL2` event for payload with length 6:
    //              -> 375 const opcode cost
    //              -> 4 * 375 topics cost (fromAddress, toAddress, selector and 1 default)
    //              -> 9 * 256 data array cost (payload length, nonce and 2 required solidity params for array)
    //         + 5000 (gas per counter decrease, fixed cost of L1 -> L2 message)
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
    // 105 range_check_builtin = 105 * 0.04 = 4.2 = ~5
    // 768_000 gas for 50 events
    //      512_000 = 10_240 * 50 (gas from 50 event keys)
    //      256_000 = 5120 * 50 (gas from 50 event data)
    // 0 l1_gas + 0 l1_data_gas + ((5) * (100 / 0.0025) + 768_000) l2 gas
    assert_gas(
        &result,
        "events_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(0),
            l2_gas: GasAmount(968_000),
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
    // (3736 steps + 14 memory holes) * 0.0025 = 9.375 ~ 10 = gas cost of steps
    // 96 = gas cost of onchain data (deploy cost)
    // 768_000 gas for 50 events
    //      512_000 = 10_240 * 50 (gas from 50 event keys)
    //      256_000 = 5120 * 50 (gas from 50 event data)
    // 0 l1_gas + 96 l1_data_gas + (10 * (100 / 0.0025) + 768_000) l2 gas
    assert_gas(
        &result,
        "event_emission_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(96),
            l2_gas: GasAmount(1_168_000),
        },
    );
}

#[test]
fn nested_call_cost_cairo_steps() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{ContractClassTrait, DeclareResultTrait, declare};
            use starknet::{ContractAddress, SyscallResult};

            #[starknet::interface]
            trait IGasCheckerProxy<TContractState> {
                fn call_other_contract(
                    self: @TContractState,
                    contract_address: ContractAddress,
                    entry_point_selector: felt252,
                    calldata: Array::<felt252>,
                ) -> SyscallResult<Span<felt252>>;
            }

            fn deploy_contract(name: ByteArray) -> ContractAddress {
                let contract = declare(name).unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                contract_address
            }

            #[test]
            fn test_call_other_contract() {
                let contract_address_a = deploy_contract("GasCheckerProxy");
                let contract_address_b = deploy_contract("GasCheckerProxy");
                let hello_starknet_address = deploy_contract("HelloStarknet");

                let dispatcher_a = IGasCheckerProxyDispatcher { contract_address: contract_address_a };
                let _ = dispatcher_a
                    .call_other_contract(
                        contract_address_b,
                        selector!("call_other_contract"),
                        array![hello_starknet_address.into(), selector!("example_function"), 0],
                    );
            }
        "#
        ),
        Contract::from_code_path(
            "HelloStarknet".to_string(),
            Path::new("tests/data/contracts/hello_starknet_for_nested_calls.cairo"),
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
    // int(1121 * 0.16) = 180 = gas cost of bitwise builtins
    // 96 * 3 = gas cost of onchain data (deploy cost)
    // ~1 gas for 1 event key
    // ~1 gas for 1 event data
    // 0 l1_gas + (96 * 3) l1_data_gas + 180 * (100 / 0.0025) + 1 * 10240 + 1 * 5120 l2 gas
    assert_gas(
        &result,
        "test_call_other_contract",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(288),
            l2_gas: GasAmount(7_215_360),
        },
    );
}

#[test]
fn nested_call_cost_in_forked_contract_cairo_steps() {
    let test = test_case!(
        formatdoc!(
            r#"
            use snforge_std::{{ContractClassTrait, DeclareResultTrait, declare}};
            use starknet::{{ContractAddress, SyscallResult}};

            #[starknet::interface]
            trait IGasCheckerProxy<TContractState> {{
                fn call_other_contract(
                    self: @TContractState,
                    contract_address: ContractAddress,
                    entry_point_selector: felt252,
                    calldata: Array::<felt252>,
                ) -> SyscallResult<Span<felt252>>;
            }}

            fn deploy_contract(name: ByteArray) -> ContractAddress {{
                let contract = declare(name).unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                contract_address
            }}

            #[test]
            #[fork(url: "{}", block_number: 861_389)]
            fn test_call_other_contract_fork() {{
                let contract_address_a = deploy_contract("GasCheckerProxy");
                let contract_address_b = deploy_contract("GasCheckerProxy");
                let hello_starknet_address: ContractAddress = 0x07f01bbebed8dfeb60944bd9273e2bd844e39b0106eb6ca05edaeee95a817c64.try_into().unwrap();

                let dispatcher_a = IGasCheckerProxyDispatcher {{ contract_address: contract_address_a }};
                let _ = dispatcher_a
                    .call_other_contract(
                        contract_address_b,
                        selector!("call_other_contract"),
                        array![hello_starknet_address.into(), selector!("example_function"), 0],
                    );
            }}
        "#,
            node_rpc_url()
        ).as_str(),
        Contract::from_code_path(
            "HelloStarknet".to_string(),
            Path::new("tests/data/contracts/hello_starknet_for_nested_calls.cairo"),
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
    // int(1121 * 0.16) = 180 = gas cost of bitwise builtins
    // 96 * 2 = gas cost of onchain data (deploy cost)
    // ~1 gas for 1 event key
    // ~1 gas for 1 event data
    // 0 l1_gas + (96 * 2) l1_data_gas + 180 * (100 / 0.0025) + 1 * 10240 + 1 * 5120 l2 gas
    assert_gas(
        &result,
        "test_call_other_contract_fork",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(192),
            l2_gas: GasAmount(7_215_360),
        },
    );
}

#[test]
fn empty_test_sierra_gas() {
    let test = test_case!(indoc!(
        r"
            #[test]
            fn empty_test() {}
        "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::SierraGas);
    assert_passed(&result);

    assert_gas(
        &result,
        "empty_test",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(0),
            l2_gas: GasAmount(13_840),
        },
    );
}

#[test]
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

    assert_gas(
        &result,
        "declare_cost_is_omitted",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(0),
            l2_gas: GasAmount(25_350),
        },
    );
}

#[test]
fn deploy_syscall_cost_sierra_gas() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{declare, DeclareResultTrait};
            use starknet::syscalls::deploy_syscall;
            #[test]
            fn deploy_syscall_cost() {
                let contract = declare("GasConstructorChecker").unwrap().contract_class().clone();
                let (address, _) = deploy_syscall(contract.class_hash, 0, array![0].span(), false).unwrap();
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
    // 151_970 = cost of 1 deploy syscall (because 1 * (1173 + 8) * 100 + (7 + 1) * 4050 + 21 * 70)
    //      -> 1 deploy syscall costs 1132 cairo steps, 7 pedersen and 18 range check builtins
    //      -> 1 calldata element costs 8 cairo steps and 1 pedersen
    //      -> 1 pedersen costs 4050, 1 range check costs 70
    //
    // 96 l1_data_gas
    // l2 gas > 151_970
    assert_gas(
        &result,
        "deploy_syscall_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(96),
            l2_gas: GasAmount(184_360),
        },
    );
}

#[test]
fn snforge_std_deploy_cost_sierra_gas() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };
            #[test]
            fn deploy_cost() {
                let contract = declare("GasConstructorChecker").unwrap().contract_class();
                let (address, _) = contract.deploy(@array![0]).unwrap();
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
    // 151_970 = cost of 1 deploy syscall (see `deploy_syscall_cost_sierra_gas` test)
    //
    // 96 l1_data_gas
    // l2 gas > 151_970
    assert_gas(
        &result,
        "deploy_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(96),
            l2_gas: GasAmount(190_800),
        },
    );
}

#[test]
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
    // Note: To calculate the gas cost of the keccak syscall, we need to include the cost of keccak round
    // https://github.com/starkware-libs/sequencer/blob/028db0341378147037b5e7236d8e136e4ca7c30d/crates/blockifier/src/execution/syscalls/syscall_executor.rs#L190
    // 10_000 = cost of 1 keccak syscall (1 * 100 * 100)
    //      -> 1 keccak syscall costs 100 cairo steps
    // 171_707 = cost of 1 keccak round syscall (136_189 + 3498 + 3980 + 28_100)
    //      -> 1 keccak builtin costs 136_189
    //      -> 6 bitwise builtin cost 6 * 583 = 3498
    //      -> 56 range check builtins cost 56 * 70 = 3980
    //      -> 281 steps cost 281 * 100 = 28_100
    //
    // l2 gas > 181_707
    assert_gas(
        &result,
        "keccak_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(0),
            l2_gas: GasAmount(226_727),
        },
    );
}

#[test]
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
    // 147_120 = cost of 1 deploy syscall (see `deploy_syscall_cost_sierra_gas` test)
    // 908_535 = 5 * 181_707 = cost of 5 keccak syscall (see `keccak_cost_sierra_gas` test)
    // 91_560 = cost of 1 call contract syscall (because 1 * 903 * 100 + 18 * 70)
    //      -> 1 call contract syscall costs 903 cairo steps and 18 range check builtins
    //      -> 1 range check costs 70
    //
    // 96 l1_data_gas
    // l2 gas > 1_147_215 (= 147_120 + 908_535 + 91_560)
    assert_gas(
        &result,
        "contract_keccak_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(96),
            l2_gas: GasAmount(1_353_025),
        },
    );
}

#[test]
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
    // 147_120 = cost of 1 deploy syscall (see `deploy_syscall_cost_sierra_gas` test)
    // 91_560 = cost of 1 call contract syscall (see `contract_keccak_cost_sierra_gas` test)
    // 10_000 = cost of 1 storage write syscall (because 1 * 96 * 100 + 1 * 70 = 9670)
    //      -> 1 storage write syscall costs 96 cairo steps and 1 range check builtin
    //      -> 1 range check costs 70
    //      -> the minimum total cost is `syscall_base_gas_cost`, which is pre-charged by the compiler (atm it is 100 * 100)
    //
    // (96 + 64 + 32 =) 192 l1_data_gas
    // l2 gas > 248_680 = (147_120 + 91_560 + 10_000)
    assert_gas(
        &result,
        "storage_write_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(192),
            l2_gas: GasAmount(289_350),
        },
    );
}

#[test]
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
    // 147_120 = cost of 1 deploy syscall (see `deploy_syscall_cost_sierra_gas` test)
    // 183_120 = 2 * 91_560 = cost of 2 call contract syscalls (see `contract_keccak_cost_sierra_gas` test)
    // 20_000 = cost of 2 storage write syscall (see `storage_write_cost_sierra_gas` test)
    //
    // 192 = (64 + 64 + 32 + 32) l1_data_gas
    // l2 gas > 350_240 (= 147_120 + 183_120 + 20_000)
    assert_gas(
        &result,
        "multiple_storage_writes_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(192),
            l2_gas: GasAmount(396_790),
        },
    );
}

#[test]
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
    // 29_524 = gas cost of l2 -> l1 message (payload length 3)
    // The calculation below covers L1 costs related to the state updates and emitting the event `LogMessageToL1`.
    // https://github.com/starkware-libs/sequencer/blob/028db0341378147037b5e7236d8e136e4ca7c30d/crates/blockifier/src/fee/resources.rs#L338-L340
    //      -> (3 + 3) * 1124 = state update costs
    //      -> 375 const opcode cost
    //      -> 3 * 375 = 1225 topics cost of `LogMessageToL1` event (fromAddress, toAddress and 1 default)
    //      -> 5 * 256 = 1280 data array cost (payload length + 2 required solidity params for array)
    //      -> 20_000 l1 storage write cost
    // 96 = gas cost of onchain data (see `deploy_syscall_cost_sierra_gas` test)
    // 147_120 = cost of 1 deploy syscall (see `deploy_syscall_cost_sierra_gas` test)
    // 91_560 = cost of 1 call contract syscall (see `contract_keccak_cost_sierra_gas` test)
    // 14_470 = cost of 1 SendMessageToL1 syscall (because 1 * 144 * 100 + 1 * 70 )
    //      -> 1 storage write syscall costs 144 cairo steps and 1 range check builtin
    //      -> 1 range check costs 70
    //
    // 29_524 l1_gas
    // 96 l1_data_gas
    // l2 gas > 253_150 (= 147_120 + 91_560 + 14_470)
    assert_gas(
        &result,
        "l1_message_cost",
        GasVector {
            l1_gas: GasAmount(29_524),
            l1_data_gas: GasAmount(96),
            l2_gas: GasAmount(293_180),
        },
    );
}

#[test]
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
    // 29_524 = gas cost of l2 -> l1 message (see `l1_message_cost_sierra_gas` test)
    // 128 = n(2) * 2 * 32
    // 64 = l(2) * 32
    //      -> l = number of class hash updates
    //      -> n = unique contracts updated
    // 294_240 = 2 * 147_120 = cost of 2 deploy syscall (see `deploy_syscall_cost_sierra_gas` test)
    // 183_120 = 2 * 91_560 = cost of 2 call contract syscalls (see `multiple_storage_writes_cost_sierra_gas` test)
    // 14_470 = cost of 1 SendMessageToL1 syscall (see `l1_message_cost_sierra_gas` test)
    //
    // 29_524 l1_gas
    // (128 + 64 =) 192 l1_data_gas
    // l2 gas > 491_830 (= 294_240 + 183_120 + 14_470)
    assert_gas(
        &result,
        "l1_message_cost_for_proxy",
        GasVector {
            l1_gas: GasAmount(29_524),
            l1_data_gas: GasAmount(192),
            l2_gas: GasAmount(557_260),
        },
    );
}

#[test]
fn events_cost_sierra_gas() {
    let test = test_case!(indoc!(
        r"
            use starknet::syscalls::emit_event_syscall;
            #[test]
            fn events_cost() {
                let mut keys = array![];
                let mut values =  array![];
                let mut i: u32 = 0;
                while i < 10 {
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
    // 102_400 = 10 * 10_240
    //      -> we emit 50 keys, each taking up 1 felt of space
    //      -> L2 gas cost for event key is 10240 gas/felt
    // 51_200 = 10 * 5120
    //      -> we emit 50 keys, each having 1 felt of data
    //      -> L2 gas cost for event data is 5120 gas/felt
    // 10_000 = cost of 1 emit event syscall (because 1 * 61 * 100 + 1 * 70 = 6170)
    //      -> 1 emit event syscall costs 61 cairo steps and 1 range check builtin
    //      -> 1 range check costs 70
    //      -> the minimum total cost is `syscall_base_gas_cost`, which is pre-charged by the compiler (atm it is 100 * 100)
    //
    // l2 gas > 163_600 (= 102_400 + 51_200 + 10_000)
    assert_gas(
        &result,
        "events_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(0),
            l2_gas: GasAmount(205_510),
        },
    );
}

#[test]
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
                dispatcher.emit_event(10);
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
    // 102_400 = event keys cost (see `events_cost_sierra_gas` test)
    // 51_200 = event data cost (see `events_cost_sierra_gas` test)
    // 10_000 = cost of 1 emit event syscall (see `events_cost_sierra_gas` test)
    // 147_120 = cost of 1 deploy syscall (see `deploy_syscall_cost_sierra_gas` test)
    // 91_560 = cost of 1 call contract syscall (see `contract_keccak_cost_sierra_gas` test)
    // 159_810 = reported consumed sierra gas
    //
    // 96 l1_data_gas
    // l2 gas > 402_280 (= 102_400 + 51_200 + 10_000 + 147_120 + 91_560)
    assert_gas(
        &result,
        "event_emission_cost",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(96),
            l2_gas: GasAmount(471_090),
        },
    );
}

#[test]
fn nested_call_cost_sierra_gas() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{ContractClassTrait, DeclareResultTrait, declare};
            use starknet::{ContractAddress, SyscallResult};
            #[starknet::interface]
            trait IGasCheckerProxy<TContractState> {
                fn call_other_contract(
                    self: @TContractState,
                    contract_address: ContractAddress,
                    entry_point_selector: felt252,
                    calldata: Array::<felt252>,
                ) -> SyscallResult<Span<felt252>>;
            }
            fn deploy_contract(name: ByteArray) -> ContractAddress {
                let contract = declare(name).unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                contract_address
            }
            #[test]
            fn test_call_other_contract() {
                let contract_address_a = deploy_contract("GasCheckerProxy");
                let contract_address_b = deploy_contract("GasCheckerProxy");
                let hello_starknet_address = deploy_contract("HelloStarknet");
                let dispatcher_a = IGasCheckerProxyDispatcher { contract_address: contract_address_a };
                let _ = dispatcher_a
                    .call_other_contract(
                        contract_address_b,
                        selector!("call_other_contract"),
                        array![hello_starknet_address.into(), selector!("example_function"), 0],
                    );
            }
        "#
        ),
        Contract::from_code_path(
            "HelloStarknet".to_string(),
            Path::new("tests/data/contracts/hello_starknet_for_nested_calls.cairo"),
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
    // 10_240 = event keys cost (see `events_cost_sierra_gas` test)
    // 5120 = event data cost (see `events_cost_sierra_gas` test)
    // 10_000 = cost of 1 emit event syscall (see `events_cost_sierra_gas` test)
    // 181_707 = cost of 1 keccak syscall (see `keccak_cost_sierra_gas` test)
    // 10_840 = cost of 1 get block hash syscall (107 * 100 + 2 * 70)
    // 441_360 = 3 * 147_120 = cost of 3 deploy syscall (see `deploy_syscall_cost_sierra_gas` test)
    // 274_680 = 3 * 91_560 = cost of 3 call contract syscall (see `contract_keccak_cost_sierra_gas` test)
    // 841_295 = cost of 1 sha256_process_block_syscall syscall (1867 * 100 + 1115 * 583 + 65 * 70)
    //
    // 288 l1_data_gas
    // l2 gas > 1_775_242 (= 10_240 + 5120 + 10_000 + 181_707 + 10_840 + 441_360 + 274_680 + 841_295)
    assert_gas(
        &result,
        "test_call_other_contract",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(288),
            l2_gas: GasAmount(1_940_292),
        },
    );
}

#[test]
fn nested_call_cost_in_forked_contract_sierra_gas() {
    let test = test_case!(
        formatdoc!(
            r#"
            use snforge_std::{{ContractClassTrait, DeclareResultTrait, declare}};
            use starknet::{{ContractAddress, SyscallResult}};
            #[starknet::interface]
            trait IGasCheckerProxy<TContractState> {{
                fn call_other_contract(
                    self: @TContractState,
                    contract_address: ContractAddress,
                    entry_point_selector: felt252,
                    calldata: Array::<felt252>,
                ) -> SyscallResult<Span<felt252>>;
            }}
            fn deploy_contract(name: ByteArray) -> ContractAddress {{
                let contract = declare(name).unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                contract_address
            }}
            #[test]
            #[fork(url: "{}", block_number: 861_389)]
            fn test_call_other_contract_fork() {{
                let contract_address_a = deploy_contract("GasCheckerProxy");
                let contract_address_b = deploy_contract("GasCheckerProxy");
                let hello_starknet_address: ContractAddress = 0x07f01bbebed8dfeb60944bd9273e2bd844e39b0106eb6ca05edaeee95a817c64.try_into().unwrap();
                let dispatcher_a = IGasCheckerProxyDispatcher {{ contract_address: contract_address_a }};
                let _ = dispatcher_a
                    .call_other_contract(
                        contract_address_b,
                        selector!("call_other_contract"),
                        array![hello_starknet_address.into(), selector!("example_function"), 0],
                    );
            }}
        "#,
        node_rpc_url()
        ).as_str(),
        Contract::from_code_path(
            "HelloStarknet".to_string(),
            Path::new("tests/data/contracts/hello_starknet_for_nested_calls.cairo"),
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
    // 10_240 = event keys cost (see `events_cost_sierra_gas` test)
    // 5120 = event data cost (see `events_cost_sierra_gas` test)
    // 10_000 = cost of 1 emit event syscall (see `events_cost_sierra_gas` test)
    // 181_707 = cost of 1 keccak syscall (see `keccak_cost_sierra_gas` test)
    // 10_840 = cost of 1 get block hash syscall (107 * 100 + 2 * 70)
    // 294_240 = 2 * 147_120 = cost of 2 deploy syscall (see `deploy_syscall_cost_sierra_gas` test)
    // 274_680 = 3 * 91_560 = cost of 3 call contract syscall (see `contract_keccak_cost_sierra_gas` test)
    // 841_295 = cost of 1 sha256_process_block_syscall syscall (1867 * 100 + 1115 * 583 + 65 * 70)
    //
    // 192 l1_data_gas
    // l2 gas > 1_628_122 (= 10_240 + 5120 + 10_000 + 181_707 + 10_840 + 294_240 + 274_680 + 841_295)
    assert_gas(
        &result,
        "test_call_other_contract_fork",
        GasVector {
            l1_gas: GasAmount(0),
            l1_data_gas: GasAmount(192),
            l2_gas: GasAmount(1_812_052),
        },
    );
}
