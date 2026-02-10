use crate::utils::runner::{Contract, assert_case_output_contains, assert_failed, assert_passed};
use crate::utils::running_tests::run_test_case;
use crate::utils::test_case;
use forge_runner::forge_config::ForgeTrackedResource;
use indoc::indoc;
use std::path::Path;

#[test]
fn deploy_at_correct_address() {
    let test = test_case!(
        indoc!(
            r#"
        use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };
        use starknet::ContractAddress;

        #[starknet::interface]
        trait IProxy<TContractState> {
            fn get_caller_address(ref self: TContractState, checker_address: ContractAddress) -> felt252;
        }

        #[test]
        fn deploy_at_correct_address() {
            let contract = declare("CheatCallerAddressChecker").unwrap().contract_class();
            let (cheat_caller_address_checker, _) = contract.deploy(@array![]).unwrap();

            let contract = declare("Proxy").unwrap().contract_class();
            let deploy_at_address = 123;

            let (contract_address, _) = contract.deploy_at(@array![], deploy_at_address.try_into().unwrap()).unwrap();
            assert(deploy_at_address == contract_address.into(), 'addresses should be the same');

            let real_address = IProxyDispatcher{ contract_address }.get_caller_address(cheat_caller_address_checker);
            assert(real_address == contract_address.into(), 'addresses should be the same');
        }
    "#
        ),
        Contract::new(
            "Proxy",
            indoc!(
                r"
                use starknet::ContractAddress;

                #[starknet::interface]
                trait IProxy<TContractState> {
                    fn get_caller_address(ref self: TContractState, checker_address: ContractAddress) -> felt252;
                }

                #[starknet::contract]
                mod Proxy {
                    use starknet::ContractAddress;
                                                    
                    #[storage]
                    struct Storage {}

                    #[starknet::interface]
                    trait ICheatCallerAddressChecker<TContractState> {
                        fn get_caller_address(ref self: TContractState) -> felt252;
                    }
                
                    #[abi(embed_v0)]
                    impl ProxyImpl of super::IProxy<ContractState> {
                        fn get_caller_address(ref self: ContractState, checker_address: ContractAddress) -> felt252 {
                            ICheatCallerAddressCheckerDispatcher{ contract_address: checker_address}.get_caller_address()
                        }
                    }
                }
            "
            )
        ),
        Contract::from_code_path(
            "CheatCallerAddressChecker".to_string(),
            Path::new("tests/data/contracts/cheat_caller_address_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn deploy_two_at_the_same_address() {
    let test = test_case!(
        indoc!(
            r#"
        use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };
        use starknet::ContractAddress;

        #[test]
        fn deploy_two_at_the_same_address() {
            let contract_address = 123;

            let contract = declare("HelloStarknet").unwrap().contract_class();
            let (real_address, _) = contract.deploy_at(@array![], contract_address.try_into().unwrap()).unwrap();
            assert(real_address.into() == contract_address, 'addresses should be the same');
            contract.deploy_at(@array![], contract_address.try_into().unwrap()).unwrap();
        }
    "#
        ),
        Contract::from_code_path(
            "HelloStarknet".to_string(),
            Path::new("tests/data/contracts/hello_starknet.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_failed(&result);
    assert_case_output_contains(
        &result,
        "deploy_two_at_the_same_address",
        "Deployment failed: contract already deployed at address 0x000000000000000000000000000000000000000000000000000000000000007b",
    );
}

#[test]
fn fail_to_deploy_at_0() {
    let test = test_case!(
        indoc!(
            r#"
        use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };

        #[test]
        fn deploy_at_0() {
            let contract = declare("HelloStarknet").unwrap().contract_class();
            contract.deploy_at(@array![], 0.try_into().unwrap()).unwrap();
        }
    "#
        ),
        Contract::from_code_path(
            "HelloStarknet".to_string(),
            Path::new("tests/data/contracts/hello_starknet.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_failed(&result);
    assert_case_output_contains(
        &result,
        "deploy_at_0",
        "Cannot deploy contract at address 0",
    );
}

#[test]
fn deploy_at_constructor_panic_catchable_via_should_panic() {
    let test = test_case!(
        indoc!(
            r#"
        use snforge_std::{ContractClassTrait, DeclareResultTrait, declare};
        use starknet::{SyscallResultTrait, ContractAddress};

        #[test]
        #[should_panic(expected: 'Initial balance cannot be 0')]
        fn deploy_at_constructor_panic_should_be_catchable() {
            let contract = declare("DeployChecker").unwrap().contract_class();
            let constructor_calldata = array![0];
            let deploy_at_address = 123;
            
            contract.deploy_at(@constructor_calldata, deploy_at_address.try_into().unwrap()).unwrap_syscall();
        }
    "#
        ),
        Contract::from_code_path(
            "DeployChecker".to_string(),
            Path::new("tests/data/contracts/deploy_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);
    assert_passed(&result);
}
