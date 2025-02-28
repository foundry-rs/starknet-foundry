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
        use snforge_std::{ declare, ContractClass, ContractClassTrait, DeclareResultTrait };
        use array::ArrayTrait;

        #[test]
        fn error_handling() {
            let contract = declare("PanickingConstructor").unwrap().contract_class();

            match contract.deploy(@ArrayTrait::new()) {
                Result::Ok(_) => panic_with_felt252('Should have panicked'),
                Result::Err(panic_data) => {
                    assert(*panic_data.at(0_usize) == 'PANIK', *panic_data.at(0_usize));
                    assert(*panic_data.at(1_usize) == 'DEJTA', *panic_data.at(1_usize));
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
        use core::clone::Clone;
        use snforge_std::{declare, test_address, DeclareResultTrait};
        use starknet::{SyscallResult, deploy_syscall};

        #[starknet::interface]
        trait IDeployChecker<T> {
            fn get_balance(self: @T) -> felt252;
            fn get_caller(self: @T) -> starknet::ContractAddress;
        }

        #[test]
        fn deploy_syscall_check() {
            let contract = declare("DeployChecker").unwrap().contract_class().clone();
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

#[test]
fn constructor_retdata_span() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use snforge_std::{ declare, ContractClass, ContractClassTrait, DeclareResultTrait };
        use array::ArrayTrait;

        #[test]
        fn constructor_retdata_span() {
            let contract = declare("ConstructorRetdata").unwrap().contract_class();

            let (_contract_address, retdata) = contract.deploy(@ArrayTrait::new()).unwrap();
            assert_eq!(retdata, array![3, 2, 3, 4].span());
        }
    "#
        ),
        Contract::new(
            "ConstructorRetdata",
            indoc!(
                r"
                #[starknet::contract]
                mod ConstructorRetdata {
                    use array::ArrayTrait;
                
                    #[storage]
                    struct Storage {}
                
                    #[constructor]
                    fn constructor(ref self: ContractState) -> Span<felt252> {
                        array![2, 3, 4].span()
                    }
                }
                "
            )
        )
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn constructor_retdata_felt() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use snforge_std::{ declare, ContractClass, ContractClassTrait, DeclareResultTrait };
        use array::ArrayTrait;

        #[test]
        fn constructor_retdata_felt() {
            let contract = declare("ConstructorRetdata").unwrap().contract_class();

            let (_contract_address, retdata) = contract.deploy(@ArrayTrait::new()).unwrap();
            assert_eq!(retdata, array![5].span());
        }
    "#
        ),
        Contract::new(
            "ConstructorRetdata",
            indoc!(
                r"
                #[starknet::contract]
                mod ConstructorRetdata {
                    use array::ArrayTrait;
                
                    #[storage]
                    struct Storage {}
                
                    #[constructor]
                    fn constructor(ref self: ContractState) -> felt252 {
                        5
                    }
                }
                "
            )
        )
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn constructor_retdata_struct() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use snforge_std::{ declare, ContractClass, ContractClassTrait, DeclareResultTrait };
        use array::ArrayTrait;

        #[test]
        fn constructor_retdata_struct() {
            let contract = declare("ConstructorRetdata").unwrap().contract_class();

            let (_contract_address, retdata) = contract.deploy(@ArrayTrait::new()).unwrap();
            assert_eq!(retdata, array![0, 6, 2, 7, 8, 9].span());
        }
    "#
        ),
        Contract::new(
            "ConstructorRetdata",
            indoc!(
                r"
                #[starknet::contract]
                mod ConstructorRetdata {
                    use array::ArrayTrait;
                
                    #[storage]
                    struct Storage {}
                
                    #[derive(Serde, Drop)]
                    struct Retdata {
                        a: felt252,
                        b: Span<felt252>,
                        c: felt252,
                    }

                    #[constructor]
                    fn constructor(ref self: ContractState) -> Option<Retdata> {
                        Option::Some(
                            Retdata {
                                a: 6,
                                b: array![7, 8].span(),
                                c: 9
                            }
                        )
                    }
                }
                "
            )
        )
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}
