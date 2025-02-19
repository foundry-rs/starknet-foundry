use indoc::indoc;
use std::path::Path;
use test_utils::runner::{assert_gas, assert_passed, Contract};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

// all calculations are based on formula from
// https://docs.starknet.io/architecture-and-concepts/network-architecture/fee-mechanism/#overall_fee

#[test]
fn declare_cost_is_omitted() {
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

    let result = run_test_case(&test);

    assert_passed(&result);
    // 1 = cost of 230 steps (because int(0.0025 * 230) = 1)
    assert_gas(&result, "declare_cost_is_omitted", 1);
}

#[test]
fn deploy_syscall_cost() {
    let test = test_case!(
        indoc!(
            r#"
            use core::clone::Clone;
            use snforge_std::{declare, DeclareResultTrait};
            use starknet::{SyscallResult, deploy_syscall};

            #[test]
            fn deploy_syscall_cost() {
                let contract = declare("GasConstructorChecker").unwrap().contract_class().clone();
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

    assert_passed(&result);
    // l = 1 (updated contract class)
    // n = 1 (unique contracts updated - in this case it's the new contract address)
    // ( l + n * 2 ) * felt_size_in_bytes(32) = 96 (total l1 cost)
    // 11 = cost of 2 keccak builtins from constructor (because int(5.12 * 2) = 11)
    assert_gas(&result, "deploy_syscall_cost", 96 + 11);
}

#[test]
fn snforge_std_deploy_cost() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };

            #[test]
            fn deploy_cost() {
                let contract = declare("GasConstructorChecker").unwrap().contract_class();
                let (address, _) = contract.deploy(@array![]).unwrap();
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

    assert_passed(&result);
    // 96 = gas cost of onchain data (deploy cost)
    // 11 = cost of 2 keccak builtins = 11 (because int(5.12 * 2) = 11)
    assert_gas(&result, "deploy_cost", 96 + 11);
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

    assert_passed(&result);
    // 6 = cost of 1 keccak builtin (because int(5.12 * 1) = 6)
    assert_gas(&result, "keccak_cost", 6);
}

#[test]
fn contract_keccak_cost() {
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

    let result = run_test_case(&test);

    assert_passed(&result);
    // 96 = cost of deploy (see snforge_std_deploy_cost test)
    // 26 = cost of 5 keccak builtins (because int(5.12 * 5) = 26)
    assert_gas(&result, "contract_keccak_cost", 96 + 26);
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

    assert_passed(&result);
    // 1 = cost of 1 range check builtin (because int(0.04 * 1) = 1)
    assert_gas(&result, "range_check_cost", 1);
}

/// Declare, deploy and function call consume 13 `range_check_builtin`s
/// `range_check` function consumes 9, so
/// overall cost will be 22 * range check builtin cost.
#[test]
fn contract_range_check_cost() {
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

    let result = run_test_case(&test);

    assert_passed(&result);
    // 96 = cost of deploy (see snforge_std_deploy_cost test)
    // 8 = cost of 191 range check builtins (because int(0.04 * 191) = 8)
    assert_gas(&result, "contract_range_check_cost", 96 + 8);
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

    assert_passed(&result);
    // 1 = cost of 1 bitwise builtin, because int(0.16 * 1) = 1
    assert_gas(&result, "bitwise_cost", 1);
}

/// We have to use 6 bitwise operations in the `bitwise` function to exceed steps cost
#[test]
fn contract_bitwise_cost() {
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

    let result = run_test_case(&test);

    assert_passed(&result);
    // 96 = cost of deploy l1 cost (see snforge_std_deploy_cost test)
    // 48 = cost of 300 bitwise builtins (because int(0.16 * 300) = 48)
    assert_gas(&result, "contract_bitwise_cost", 96 + 48);
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

    assert_passed(&result);
    // 1 = cost of 1 pedersen builtin (because int(0.16 * 1) = 1)
    assert_gas(&result, "pedersen_cost", 1);
}

/// We have to use 12 pedersen operations in the `pedersen` function to exceed steps cost
#[test]
fn contract_pedersen_cost() {
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

    let result = run_test_case(&test);

    assert_passed(&result);
    // 96 = cost of deploy (see snforge_std_deploy_cost test)
    // 7 = cost of 86 pedersen builtins (because int(0.08 * 86) = 7)
    assert_gas(&result, "contract_pedersen_cost", 96 + 7);
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

    assert_passed(&result);
    // 1 = cost of 1 poseidon builtin (because int(0.08 * 1) = 1)
    assert_gas(&result, "poseidon_cost", 1);
}

/// We have to use 12 poseidon operations in the `poseidon` function to exceed steps cost
#[test]
fn contract_poseidon_cost() {
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

    let result = run_test_case(&test);

    assert_passed(&result);
    // 96 = cost of deploy (see snforge_std_deploy_cost test)
    // 13 = cost of 160 poseidon builtins (because int(0.08 * 160) = 13)
    assert_gas(&result, "contract_poseidon_cost", 96 + 13);
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

    assert_passed(&result);
    // 3 = cost of 1 ec_op builtin (because int(2.56 * 1) = 3)
    assert_gas(&result, "ec_op_cost", 3);
}

#[test]
fn contract_ec_op_cost() {
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

    let result = run_test_case(&test);

    assert_passed(&result);
    // 96 = cost of deploy (see snforge_std_deploy_cost test)
    // 26 = cost of 10 ec_op builtins (because int(2.56 * 10) = 26)
    assert_gas(&result, "contract_ec_op_cost", 96 + 26);
}

#[test]
fn storage_write_cost() {
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

    let result = run_test_case(&test);

    assert_passed(&result);
    // 2576 * 0.0025 = 6.44 ~ 7 = gas cost of steps
    // 96 = gas cost of deployment
    // storage_updates(1) * 2 * 32 = 64
    // storage updates from zero value(1) * 32 = 32 (https://community.starknet.io/t/starknet-v0-13-4-pre-release-notes/115257#p-2358763-da-costs-27)
    assert_gas(&result, "storage_write_cost", 7 + 96 + 64 + 32);
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


        #[test]
        fn storage_write_from_test_cost() {
            let mut state = Contract::contract_state_for_testing();
            state.balance.write(10);
        }
    "
    ),);

    let result = run_test_case(&test);

    assert_passed(&result);
    // 173 * 0.0025 = 0.4325 ~ 1 = gas cost of steps
    // n = unique contracts updated
    // m = values updated
    // So, as per formula:
    // n(1) * 2 * 32 = 64
    // m(1) * 2 * 32 = 64
    // storage updates from zero value(1) * 32 = 32 (https://community.starknet.io/t/starknet-v0-13-4-pre-release-notes/115257#p-2358763-da-costs-27)
    assert_gas(&result, "storage_write_from_test_cost", 1 + 64 + 64 + 32);
}

#[test]
fn multiple_storage_writes_cost() {
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

    let result = run_test_case(&test);

    assert_passed(&result);
    // (3598 + 9 memory holes) * 0.0025 = 9.0175 ~ 10 = gas cost of steps
    // l = number of class hash updates
    // n = unique contracts updated
    // m = unique(!) values updated
    // So, as per formula:
    // n(1) * 2 * 32 = 64
    // m(1) * 2 * 32 = 64
    // l(1) * 32 = 32
    // storage updates from zero value(1) * 32 = 32 (https://community.starknet.io/t/starknet-v0-13-4-pre-release-notes/115257#p-2358763-da-costs-27)
    assert_gas(
        &result,
        "multiple_storage_writes_cost",
        10 + 64 + 64 + 32 + 32,
    );
}

#[test]
fn l1_message_cost() {
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

    let result = run_test_case(&test);

    assert_passed(&result);
    // 2614 * 0.0025 = 6.535 ~ 7 = gas cost of steps
    // 96 = gas cost of deployment
    // 29524 = gas cost of onchain data
    assert_gas(&result, "l1_message_cost", 7 + 96 + 29524);
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

    assert_passed(&result);
    // 224 * 0.0025 = 0.56 ~ 1 = gas cost of steps
    // 26764 = gas cost of onchain data
    assert_gas(&result, "l1_message_from_test_cost", 1 + 26764);
}

#[test]
fn l1_message_cost_for_proxy() {
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

    let result = run_test_case(&test);

    assert_passed(&result);
    // 5205 * 0.0025 = 13.0125 ~ 14 = gas cost of steps
    // l = number of class hash updates
    // n = unique contracts updated
    // So, as per formula:
    // n(2) * 2 * 32 = 128
    // l(2) * 32 = 64
    // 29524 = gas cost of message
    assert_gas(&result, "l1_message_cost_for_proxy", 14 + 128 + 64 + 29524);
}

#[test]
fn l1_handler_cost() {
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

    let result = run_test_case(&test);
    assert_passed(&result);
    // 96 = gas cost of onchain data (deploy cost)
    // int(5.12 * 4) = 21 = keccak cost from l1 handler
    // in this test, l1_handler_payload_size = 6
    // 15923 = 12251 (gas used for processing L1<>L2 messages on L1) + 3672 (SHARP gas, 6 * 612)
    // 12251 = 3072 (6 * 512, 512 is gas per memory word) +
    //         + 4179 (result of get_consumed_message_to_l2_emissions_cost(6) which is get_event_emission_cost(3, 3 + 6) = 375 + (3 + 1) * 375 + 9 * 256) +
    //         + 0 +
    //         + 5000 (1 * 5000, 5000 is gas per counter decrease)
    //
    assert_gas(&result, "l1_handler_cost", 96 + 21 + 15923);
}

#[test]
fn events_cost() {
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

    let result = run_test_case(&test);

    assert_passed(&result);
    // 156 range_check_builtin ~= 7
    // 6 gas for 50 event values
    // ~13 gas for 50 event keys
    assert_gas(&result, "events_cost", 7 + 6 + 13);
}

#[test]
fn events_contract_cost() {
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

    let result = run_test_case(&test);
    assert_passed(&result);
    // 4119 * 0.0025 = 10.2975 ~ 11 = gas cost of steps
    // 96 = gas cost of onchain data (deploy cost)
    // 6 gas for 50 event values
    // ~13 gas for 50 event keys
    assert_gas(&result, "event_emission_cost", 11 + 96 + 6 + 13);
}
