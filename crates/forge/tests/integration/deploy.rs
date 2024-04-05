use indoc::indoc;
use std::path::Path;
use test_utils::runner::{assert_passed, Contract};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
fn error_handling() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use snforge_std::{ declare, ContractClass, ContractClassTrait };
        use array::ArrayTrait;

        #[test]
        fn error_handling() {
            let contract = declare("PanickingConstructor").unwrap();

            match contract.deploy(@ArrayTrait::new()) {
                Result::Ok(_) => panic_with_felt252('Should have panicked'),
                Result::Err(x) => {
                    assert(*x.panic_data.at(0_usize) == 'PANIK', *x.panic_data.at(0_usize));
                    assert(*x.panic_data.at(1_usize) == 'DEJTA', *x.panic_data.at(1_usize));
                }
            }
        }
    "#
        ),
        Contract::from_code_path(
            "PanickingConstructor".to_string(),
            Path::new("tests/data/contracts/panicking_constructor.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn deploy_syscall_check() {
    let test = test_case!(
        indoc!(
            r#"
        use snforge_std::{declare, test_address};
        use starknet::{SyscallResult, deploy_syscall};
        
        #[starknet::interface]
        trait IDeployChecker<T> {
            fn get_balance(self: @T) -> felt252;
            fn get_caller(self: @T) -> starknet::ContractAddress;
        }

        #[test]
        fn deploy_syscall_check() {
            let contract = declare("DeployChecker").unwrap();
            let salt = 1;
            let calldata = array![10];
        
            let (contract_address, span) = deploy_syscall(contract.class_hash, salt, calldata.span(), false).unwrap();
            assert(*span[0] == test_address().into() && *span[1] == 10, 'constructor return mismatch');
            
            let dispatcher = IDeployCheckerDispatcher { contract_address };
            assert(dispatcher.get_balance() == 10, 'balance mismatch');
            assert(dispatcher.get_caller() == test_address(), 'caller mismatch');

            let (contract_address_from_zero, _) = deploy_syscall(contract.class_hash, salt, calldata.span(), true).unwrap();
            assert(contract_address != contract_address_from_zero, 'deploy from zero no effect');
        }
    "#
        ),
        Contract::from_code_path(
            "DeployChecker".to_string(),
            Path::new("tests/data/contracts/deploy_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}
