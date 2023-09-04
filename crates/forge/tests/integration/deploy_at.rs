use crate::integration::common::runner::Contract;
use crate::integration::common::running_tests::run_test_case;
use crate::{assert_passed, test_case};
use indoc::indoc;
use std::path::Path;

#[test]
fn deploy_at_correct_address() {
    let test = test_case!(
        indoc!(
            r#"
        use snforge_std::{ declare, ContractClass, ContractClassTrait };
        use array::ArrayTrait;
        use starknet::ContractAddress;
        
        #[starknet::interface]
        trait IProxy<TContractState> {
            fn get_caller_address(ref self: TContractState, checker_address: ContractAddress) -> felt252;
        }

        #[test]
        fn test_deploy_error_handling() {
            let contract = declare('PrankChecker');
            let prank_checker = contract.deploy(@ArrayTrait::new()).unwrap();
        
            let contract = declare('Proxy');
            let deploy_at_address = 123;

            let contract_address = contract.deploy_at(@ArrayTrait::new(), deploy_at_address.try_into().unwrap()).unwrap();
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
