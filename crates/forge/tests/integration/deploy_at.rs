use indoc::indoc;
use std::path::Path;
use utils::runner::Contract;
use utils::running_tests::run_test_case;
use utils::{assert_case_output_contains, assert_failed, assert_passed, test_case};

#[test]
fn deploy_at_correct_address() {
    let test = test_case!(
        indoc!(
            r#"
        use snforge_std::{ declare, ContractClassTrait };
        use starknet::ContractAddress;
        
        #[starknet::interface]
        trait IProxy<TContractState> {
            fn get_caller_address(ref self: TContractState, checker_address: ContractAddress) -> felt252;
        }

        #[test]
        fn test_deploy_at() {
            let contract = declare('PrankChecker');
            let prank_checker = contract.deploy(@array![]).unwrap();
        
            let contract = declare('Proxy');
            let deploy_at_address = 123;

            let contract_address = contract.deploy_at(@array![], deploy_at_address.try_into().unwrap()).unwrap();
            assert(deploy_at_address == contract_address.into(), 'addresses should be the same');
            
            let real_address = IProxyDispatcher{ contract_address }.get_caller_address(prank_checker);
            assert(real_address == contract_address.into(), 'addresses should be the same');
        }
    "#
        ),
        Contract::new(
            "Proxy",
            indoc!(
                r#"
                #[starknet::contract]
                mod Proxy {
                    use starknet::ContractAddress;
                                                    
                    #[storage]
                    struct Storage {}
                    
                    #[starknet::interface]
                    trait IPrankChecker<TContractState> {
                        fn get_caller_address(ref self: TContractState) -> felt252;
                    }
                
                    #[external(v0)]
                    fn get_caller_address(ref self: ContractState, checker_address: ContractAddress) -> felt252 {
                        IPrankCheckerDispatcher{ contract_address: checker_address}.get_caller_address()
                    }
                }
            "#
            )
        ),
        Contract::from_code_path(
            "PrankChecker".to_string(),
            Path::new("tests/data/contracts/prank_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn deploy_two_at_the_same_address() {
    let test = test_case!(
        indoc!(
            r#"
        use snforge_std::{ declare, ContractClassTrait };
        use starknet::ContractAddress;

        #[test]
        fn test_deploy_two_at_the_same_address() {
            let contract_address = 123;
        
            let contract = declare('HelloStarknet');
            let real_address = contract.deploy_at(@array![], contract_address.try_into().unwrap()).unwrap();
            assert(real_address.into() == contract_address, 'addresses should be the same');
            let real_address2 = contract.deploy_at(@array![], contract_address.try_into().unwrap()).unwrap();
        }
    "#
        ),
        Contract::from_code_path(
            "HelloStarknet".to_string(),
            Path::new("tests/data/contracts/hello_starknet.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_failed!(result);
    assert_case_output_contains!(
        result,
        "test_deploy_two_at_the_same_address",
        "Address is already taken"
    );
}

#[test]
fn deploy_at_error_handling() {
    let test = test_case!(
        indoc!(
            r#"
        use array::ArrayTrait;
        use snforge_std::{ declare, ContractClassTrait, RevertedTransaction };
        use starknet::ContractAddress;

        #[test]
        fn test_deploy_at_error_handling() {
            let contract_address = 123;
        
            let contract = declare('PanickingConstructor');
            match contract.deploy_at(@array![], contract_address.try_into().unwrap()) {
                Result::Ok(_) => panic_with_felt252('shouldve panicked'),
                Result::Err(RevertedTransaction { panic_data }) => {
                    assert(*panic_data.at(0) == 'PANIK', 'wrong 1st panic datum');
                    assert(*panic_data.at(1) == 'DEJTA', 'wrong 2nd panic datum');
                },
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
