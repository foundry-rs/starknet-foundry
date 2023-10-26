use crate::integration::common::runner::Contract;
use crate::integration::common::running_tests::run_test_case;
use crate::{assert_passed, test_case};
use indoc::indoc;
use std::path::Path;

#[test]
fn error_handling() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use snforge_std::{ declare, ContractClass, ContractClassTrait };
        use array::ArrayTrait;

        #[test]
        fn test_deploy_error_handling() {
            let contract = declare('PanickingConstructor');

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

    assert_passed!(result);
}

#[test]
fn deploy_syscall() {
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
        fn test_deploy_syscall() {
            let contract = declare('DeployChecker');
            let salt = 1;
            let calldata = array![10];
        
            let (contract_address, span) = deploy_syscall(contract.class_hash, salt, calldata.span(), false).unwrap();
            assert(*span[0] == test_address().into() && *span[1] == 10, 'constructor return missmatch');
            
            let dispatcher = IDeployCheckerDispatcher { contract_address };
            assert(dispatcher.get_balance() == 10, 'balance missmatch');
            assert(dispatcher.get_caller() == test_address(), 'caller missmatch');

            let (contract_address_from_zero, span) = deploy_syscall(contract.class_hash, salt, calldata.span(), true).unwrap();
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

    assert_passed!(result);
}
