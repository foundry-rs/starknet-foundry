use blockifier::execution::deprecated_syscalls::DeprecatedSyscallSelector::StorageWrite;
use indoc::indoc;
use std::path::Path;
use test_utils::runner::Contract;
use test_utils::running_tests::run_test_case;
use test_utils::{assert_passed, assert_syscalls, test_case};

#[test]
fn syscall_count() {
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
