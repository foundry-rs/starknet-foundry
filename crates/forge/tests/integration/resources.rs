use blockifier::execution::deprecated_syscalls::DeprecatedSyscallSelector::{
    Deploy, EmitEvent, GetBlockHash, GetExecutionInfo, Keccak, SendMessageToL1, StorageRead,
    StorageWrite,
};
use indoc::indoc;
use std::path::Path;
use test_utils::runner::Contract;
use test_utils::running_tests::run_test_case;
use test_utils::{assert_builtins, assert_passed, assert_syscalls, test_case};

#[test]
fn builtins_count() {
    let test = test_case!(indoc!(
        r"
            #[test]
            fn range_check() {
                assert(1_u8 >= 1_u8, 'error message');
            }

            #[test]
            fn bitwise() {
                let _bitwise = 1_u8 & 1_u8;
                assert(1 == 1, 'error message');
            }

            #[test]
            fn pedersen() {
                core::pedersen::pedersen(1, 2);
                assert(1 == 1, 'error message');
            }

            #[test]
            fn poseidon() {
                core::poseidon::hades_permutation(0, 0, 0);
                assert(1 == 1, 'error message');
            }

            #[test]
            fn ec_op() {
                let ec_point = core::ec::EcPointTrait::new_from_x(1).unwrap();
                core::ec::EcPointTrait::mul(ec_point, 2);
                assert(1 == 1, 'error message');
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed!(result);

    // No ECDSA and Keccak builtins
    assert_builtins!(result, "range_check", "range_check_builtin", 1);
    assert_builtins!(result, "bitwise", "bitwise_builtin", 1);
    assert_builtins!(result, "pedersen", "pedersen_builtin", 1);
    assert_builtins!(result, "poseidon", "poseidon_builtin", 1);
    assert_builtins!(result, "ec_op", "ec_op_builtin", 1);
}

#[test]
fn syscalls_count() {
    let test = test_case!(
        indoc!(
            r##"
            use starknet::syscalls::{
                call_contract_syscall, keccak_syscall, deploy_syscall, get_block_hash_syscall, emit_event_syscall,
                send_message_to_l1_syscall, get_execution_info_syscall, get_execution_info_v2_syscall,
                SyscallResult
            };
            use starknet::SyscallResultTrait;
            use snforge_std::{declare, ContractClass, ContractClassTrait};
            
            #[test]
            fn keccak() {
                let input = array![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17];
                keccak_syscall(input.span()).unwrap_syscall();
            }
            
            #[test]
            fn deploy() {
                let contract = declare('HelloStarknet');
                deploy_syscall(contract.class_hash, 0, array![].span(), false).unwrap_syscall();
            }
            
            #[test]
            fn storage_read() {
                let contract = declare('HelloStarknet');
                let (address, _) = deploy_syscall(contract.class_hash, 0, array![].span(), false)
                    .unwrap_syscall();
            
                call_contract_syscall(address, selector!("get_balance"), array![].span()).unwrap_syscall();
            }
            
            #[test]
            fn storage_write() {
                let contract = declare('HelloStarknet');
                let (address, _) = deploy_syscall(contract.class_hash, 0, array![].span(), false)
                    .unwrap_syscall();
            
                call_contract_syscall(address, selector!("increase_balance"), array![123].span())
                    .unwrap_syscall();
            }
            
            #[test]
            fn get_block_hash() {
                get_block_hash_syscall(1).unwrap_syscall();
            }
            
            #[test]
            fn get_execution_info() {
                get_execution_info_syscall().unwrap_syscall();
            }
            
            #[test]
            fn get_execution_info_v2() {
                get_execution_info_v2_syscall().unwrap_syscall();
            }
            
            #[test]
            fn send_message_to_l1() {
                send_message_to_l1_syscall(1, array![1].span()).unwrap_syscall();
            }
            
            #[test]
            fn emit_event() {
                emit_event_syscall(array![1].span(), array![2].span()).unwrap_syscall();
            }
        "##
        ),
        Contract::from_code_path(
            "HelloStarknet".to_string(),
            Path::new("tests/data/contracts/hello_starknet.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);

    assert_syscalls!(result, "keccak", Keccak, 1);
    assert_syscalls!(result, "deploy", Deploy, 1);
    assert_syscalls!(result, "storage_read", StorageRead, 1);
    assert_syscalls!(result, "storage_write", StorageRead, 1);
    assert_syscalls!(result, "get_block_hash", GetBlockHash, 1);
    assert_syscalls!(result, "get_execution_info", GetExecutionInfo, 1);
    assert_syscalls!(result, "get_execution_info_v2", GetExecutionInfo, 1);
    assert_syscalls!(result, "send_message_to_l1", SendMessageToL1, 1);
    assert_syscalls!(result, "emit_event", EmitEvent, 1);
}

#[test]
fn accumulate_syscalls() {
    let test = test_case!(
        indoc!(
            r"
            use snforge_std::{ declare, ContractClassTrait };

            #[starknet::interface]
            trait IGasChecker<TContractState> {
                fn change_balance(ref self: TContractState, new_balance: u64);
            }

            #[test]
            fn single_write() {
                let contract = declare('GasChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IGasCheckerDispatcher { contract_address };

                dispatcher.change_balance(1);
            }

            #[test]
            fn double_write() {
                let contract = declare('GasChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IGasCheckerDispatcher { contract_address };

                dispatcher.change_balance(1);
                dispatcher.change_balance(2);
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
    assert_syscalls!(result, "single_write", StorageWrite, 1);
    assert_syscalls!(result, "double_write", StorageWrite, 2);
}
