use crate::utils::runner::{Contract, assert_passed};
use crate::utils::running_tests::run_test_case;
use crate::utils::test_case;
use forge_runner::forge_config::ForgeTrackedResource;
use indoc::indoc;

#[test]
fn storage_is_reverted_in_test_call() {
    let test = test_case!(
        indoc! {
            r#"
        use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };

        #[starknet::interface]
        trait IContract<TContractState> {
            fn read_storage(self: @TContractState) -> felt252;
            fn write_storage(ref self: TContractState, value: felt252);
            fn write_storage_and_panic(ref self: TContractState, value: felt252);
        }

        #[test]
        #[feature("safe_dispatcher")]
        fn test_call_storage_is_reverted() {
            let contract = declare("Contract").unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@array![]).unwrap();
            let dispatcher = IContractSafeDispatcher { contract_address }; 

            dispatcher.write_storage(5).unwrap();
            // Make sure storage value was written correctly
            let storage = dispatcher.read_storage().unwrap();
            assert_eq!(storage, 5, "Incorrect storage value");
            
            // Call storage modification and handle panic
            match dispatcher.write_storage_and_panic(11) {
                Result::Ok(_) => panic!("Should have panicked"),
                Result::Err(_) => {
                    // handled
                },
            }
            
            // Check storage change was reverted
            let storage = dispatcher.read_storage().unwrap();
            assert_eq!(storage, 5, "Storage was not reverted");
        }
            "#
        },
        Contract::from_code_path("Contract", "tests/data/contracts/reverts_contract.cairo")
            .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::SierraGas);
    assert_passed(&result);
}

#[test]
fn storage_is_reverted_in_proxy_call() {
    let test = test_case!(
        indoc! {
            r#"
        use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };

        #[starknet::interface]
        trait IProxy<TContractState> {
            fn read_storage(self: @TContractState) -> felt252;
            fn write_storage(ref self: TContractState, value: felt252);
            fn write_storage_and_panic(ref self: TContractState, value: felt252);
        }

        #[test]
        #[feature("safe_dispatcher")]
        fn test_call_storage_is_reverted() {
            let contract = declare("Contract").unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@array![]).unwrap();

            let contract_proxy = declare("Proxy").unwrap().contract_class();
            let mut calldata = array![];
            contract_address.serialize(ref calldata);
            let (contract_address_proxy, _) = contract_proxy.deploy(@calldata).unwrap();

            let dispatcher = IProxySafeDispatcher { contract_address: contract_address_proxy };

            dispatcher.write_storage(5).unwrap();
            // Make sure storage value was written correctly
            let storage = dispatcher.read_storage().unwrap();
            assert_eq!(storage, 5, "Incorrect storage value");

            // Try modifying storage and handle panic
            match dispatcher.write_storage_and_panic(1) {
                Result::Ok(_) => panic!("Should have panicked"),
                Result::Err(_panic_data) => {
                    // handled
                },
            }

            // Check storage change was reverted
            let storage = dispatcher.read_storage().unwrap();
            assert_eq!(storage, 5, "Storage was not reverted");
        }
            "#
        },
        Contract::from_code_path("Proxy", "tests/data/contracts/reverts_proxy.cairo").unwrap(),
        Contract::from_code_path("Contract", "tests/data/contracts/reverts_contract.cairo")
            .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::SierraGas);
    assert_passed(&result);
}

#[test]
fn storage_is_reverted_in_safe_proxy_call() {
    let test = test_case!(
        indoc! {
            r#"
        use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };

        #[starknet::interface]
        trait ISafeProxy<TContractState> {
            fn read_storage(self: @TContractState) -> felt252;
            fn write_storage(ref self: TContractState, value: felt252);
            fn call_write_storage_and_handle_panic(ref self: TContractState, value: felt252);
        }

        #[test]
        #[feature("safe_dispatcher")]
        fn test_call_storage_is_reverted() {
            let contract = declare("Contract").unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@array![]).unwrap();

            let contract_proxy = declare("Proxy").unwrap().contract_class();
            let mut calldata = array![];
            contract_address.serialize(ref calldata);
            let (contract_address_proxy, _) = contract_proxy.deploy(@calldata).unwrap();

            let dispatcher = ISafeProxyDispatcher { contract_address: contract_address_proxy };

            dispatcher.write_storage(5);
            // Make sure storage value was written correctly
            let storage = dispatcher.read_storage();
            assert_eq!(storage, 5, "Incorrect storage value");

            dispatcher.call_write_storage_and_handle_panic(123);

            // Check storage change was reverted
            let storage = dispatcher.read_storage();
            assert_eq!(storage, 5, "Storage was not reverted");
        }
            "#
        },
        Contract::from_code_path("Proxy", "tests/data/contracts/reverts_proxy.cairo").unwrap(),
        Contract::from_code_path("Contract", "tests/data/contracts/reverts_contract.cairo")
            .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::SierraGas);
    assert_passed(&result);
}

#[test]
fn storage_is_reverted_in_inner_call() {
    let test = test_case!(
        indoc! {
            r#"
        use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };

        #[starknet::interface]
        trait ICaller<TContractState> {
            fn call(ref self: TContractState);
        }

        #[test]
        #[feature("safe_dispatcher")]
        fn test_call_storage_is_reverted() {
            let contract = declare("Contract").unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@array![]).unwrap();

            let contract_proxy = declare("Caller").unwrap().contract_class();
            let mut calldata = array![];
            contract_address.serialize(ref calldata);
            let (contract_address_caller, _) = contract_proxy.deploy(@calldata).unwrap();

            let dispatcher = ICallerDispatcher { contract_address: contract_address_caller };
            dispatcher.call();
        }
            "#
        },
        Contract::from_code_path("Caller", "tests/data/contracts/reverts_caller.cairo").unwrap(),
        Contract::from_code_path("Contract", "tests/data/contracts/reverts_contract.cairo")
            .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::SierraGas);
    assert_passed(&result);
}

#[test]
fn storage_is_reverted_in_library_call() {
    let test = test_case!(
        indoc! {
            r#"
        use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };
        use starknet::ContractAddress;

        #[starknet::interface]
        trait ILibraryProxy<TContractState> {
            fn library_read_storage(self: @TContractState, address: ContractAddress) -> felt252;
            fn library_write_storage(self: @TContractState, address: ContractAddress, value: felt252);
            fn library_write_storage_and_panic(self: @TContractState, address: ContractAddress, value: felt252);
        }

        #[test]
        #[feature("safe_dispatcher")]
        fn test_library_call_storage_is_reverted() {
            let contract = declare("Contract").unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@array![]).unwrap();

            let contract_proxy = declare("Proxy").unwrap().contract_class();

            let dispatcher = ILibraryProxySafeLibraryDispatcher { class_hash: *contract_proxy.class_hash };

            dispatcher.library_write_storage(contract_address, 5).unwrap();
            // Make sure storage value was written correctly
            let storage = dispatcher.library_read_storage(contract_address).unwrap();
            assert_eq!(storage, 5, "Incorrect storage value");

            // Call storage modification and handle panic
            match dispatcher.library_write_storage_and_panic(contract_address, 11) {
                Result::Ok(_) => panic!("Should have panicked"),
                Result::Err(_) => {
                    // handled
                },
            }

            // Check storage change was reverted
            let storage = dispatcher.library_read_storage(contract_address).unwrap();
            assert_eq!(storage, 5, "Storage was not reverted");
        }
            "#
        },
        Contract::from_code_path("Contract", "tests/data/contracts/reverts_contract.cairo")
            .unwrap(),
        Contract::from_code_path("Proxy", "tests/data/contracts/reverts_proxy.cairo").unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::SierraGas);
    assert_passed(&result);
}
