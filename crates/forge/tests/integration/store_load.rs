use indoc::{formatdoc, indoc};
use std::path::Path;
use test_utils::assert_passed;
use test_utils::runner::Contract;
use test_utils::running_tests::run_test_case;

#[test]
fn store_load_simple() {
    let test = test_utils::test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::ContractAddress;
            use snforge_std::{ declare, ContractClassTrait, store, load };
            
            
            #[starknet::interface]
            trait IHelloStarknet<TContractState> {
                fn get_balance(ref self: TContractState) -> felt252;
                fn increase_balance(ref self: TContractState, amount: felt252);
            }
            
            fn deploy_contract() -> IHelloStarknetDispatcher {
                let contract = declare('HelloStarknet');
                let contract_address = contract.deploy(@array![]).unwrap();
                IHelloStarknetDispatcher { contract_address }
            }

            #[test]
            fn store_balance() {
                let deployed = deploy_contract();
                store(deployed.contract_address, selector!("balance"), array![420].span());
                
                let stored_balance = deployed.get_balance();
                assert(stored_balance == 420, 'wrong balance stored');
            }
            
            #[test]
            fn load_balance() {
                let deployed = deploy_contract();
                deployed.increase_balance(421);
                
                let loaded = load(deployed.contract_address, selector!("balance"), 1);
                assert(*loaded.at(0) == 421, 'wrong balance stored');
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

    assert_passed!(result);
}

#[test]
fn store_load_structure() {
    let test = test_utils::test_case!(
        indoc!(
            r#"
            use traits::Into;
            use serde::Serde;
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::ContractAddress;
            use snforge_std::{ declare, ContractClassTrait, store, load };
            
            #[derive(Serde, Copy, Drop, starknet::Store)]
            struct NestedStructure {
                c: felt252
            }
            #[derive(Serde, Copy, Drop, starknet::Store)]
            struct StoredStructure {
                a: felt252,
                b: NestedStructure,
            }
            
            impl ToSerialized of Into<StoredStructure, Span<felt252>> {
                fn into(self: StoredStructure) -> Span<felt252> {
                    let mut serialized_struct: Array<felt252> = array![];
                    self.serialize(ref serialized_struct);
                    serialized_struct.span()
                }
            }
            
            #[starknet::interface]
            trait IStorageTester<TContractState> {
                fn insert_structure(ref self: TContractState, value: StoredStructure);
                fn read_structure(self: @TContractState) -> StoredStructure;
            }
            
            fn deploy_contract() -> IStorageTesterDispatcher {
                let contract = declare('StorageTester');
                let contract_address = contract.deploy(@array![]).unwrap();
                IStorageTesterDispatcher { contract_address }
            }

            #[test]
            fn store_structure() {
                let deployed = deploy_contract();
                let stored_structure = StoredStructure { a: 123, b: NestedStructure { c: 420 } };

                store(deployed.contract_address, selector!("structure"), stored_structure.into());
                
                let stored_structure = deployed.read_structure();
                assert(stored_structure.a == 123, 'wrong stored_structure.a'); 
                assert(stored_structure.b.c == 420, 'wrong stored_structure.b.c');
            }
            
            #[test]
            fn load_structure() {
                let deployed = deploy_contract();
                let stored_structure = StoredStructure { a: 123, b: NestedStructure { c: 420 } };
                
                deployed.insert_structure(stored_structure);
                
                let loaded = load(deployed.contract_address, selector!("structure"), 2);
                assert(loaded == stored_structure.into(), 'wrong structure stored');
            }
        "#
        ),
        Contract::from_code_path(
            "StorageTester".to_string(),
            Path::new("tests/data/contracts/storage_tester.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn store_load_felt_to_structure() {
    let test = test_utils::test_case!(
        indoc!(
            r#"
            use traits::Into;
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::ContractAddress;
            use snforge_std::{ declare, ContractClassTrait, store, load, map_entry_address };
            
            #[derive(Serde, Copy, Drop, starknet::Store)]
            struct NestedStructure {
                c: felt252
            }
            #[derive(Serde, Copy, Drop, starknet::Store)]
            struct StoredStructure {
                a: felt252,
                b: NestedStructure,
            }
            
            impl ToSerialized of Into<StoredStructure, Span<felt252>> {
                fn into(self: StoredStructure) -> Span<felt252> {
                    let mut serialized_struct: Array<felt252> = array![];
                    self.serialize(ref serialized_struct);
                    serialized_struct.span()
                }
            }
            
            #[starknet::interface]
            trait IStorageTester<TContractState> {
                fn insert_felt_to_structure(ref self: TContractState, key: felt252, value: StoredStructure);
                fn read_felt_to_structure(self: @TContractState, key: felt252) -> StoredStructure;
            }
            
            fn deploy_contract() -> IStorageTesterDispatcher {
                let contract = declare('StorageTester');
                let contract_address = contract.deploy(@array![]).unwrap();
                IStorageTesterDispatcher { contract_address }
            }

            #[test]
            fn store_felt_to_structure() {
                let deployed = deploy_contract();
                let stored_structure = StoredStructure { a: 123, b: NestedStructure { c: 420 } };
                
                store(
                    deployed.contract_address, 
                    map_entry_address(selector!("felt_to_structure"), array![421]), 
                    stored_structure.into(),
                );
                
                let read_structure = deployed.read_felt_to_structure(421);
                assert(read_structure.a == stored_structure.a, 'wrong stored_structure.a'); 
                assert(read_structure.b.c == stored_structure.b.c, 'wrong stored_structure.b.c');
            }
            
            #[test]
            fn load_felt_to_structure() {
                let deployed = deploy_contract();
                let stored_structure = StoredStructure { a: 123, b: NestedStructure { c: 420 } };

                deployed.insert_felt_to_structure(421, stored_structure);
                
                let loaded = load(
                    deployed.contract_address, 
                    map_entry_address(selector!("felt_to_structure"), array![421]), 
                    2
                );
                assert(loaded == stored_structure.into(), 'wrong structure stored');
            }
        "#
        ),
        Contract::from_code_path(
            "StorageTester".to_string(),
            Path::new("tests/data/contracts/storage_tester.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn store_load_structure_to_felt() {
    let test = test_utils::test_case!(
        indoc!(
            r#"
            use traits::Into;
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::ContractAddress;
            use snforge_std::{ declare, ContractClassTrait, store, load, map_entry_address };
            
            #[derive(Serde, Copy, Drop, starknet::Store, LegacyHash)]
            struct NestedKey {
                c: felt252
            }
            #[derive(Serde, Copy, Drop, starknet::Store, LegacyHash)]
            struct StructuredKey {
                a: felt252,
                b: NestedKey,
            }
            
            impl ToSerialized of Into<StructuredKey, Array<felt252>> {
                fn into(self: StructuredKey) -> Array<felt252> {
                    let mut serialized_struct: Array<felt252> = array![];
                    self.serialize(ref serialized_struct);
                    serialized_struct
                }
            }

            #[starknet::interface]
            trait IStorageTester<TContractState> {
                fn insert_structure_to_felt(ref self: TContractState, key: StructuredKey, value: felt252);
                fn read_structure_to_felt(self: @TContractState, key: StructuredKey) -> felt252;
            }
            
            fn deploy_contract() -> IStorageTesterDispatcher {
                let contract = declare('StorageTester');
                let contract_address = contract.deploy(@array![]).unwrap();
                IStorageTesterDispatcher { contract_address }
            }

            #[test]
            fn store_structure_to_felt() {
                let deployed = deploy_contract();
                let map_key = StructuredKey {a: 420, b: NestedKey { c: 421 }};
                store(
                    deployed.contract_address, 
                    map_entry_address(selector!("structure_to_felt"), map_key.into()), 
                    array![123].span()
                );
                
                let stored_felt = deployed.read_structure_to_felt(map_key);
                assert(stored_felt == 123, 'wrong stored_felt'); 
            }
            
            #[test]
            fn load_structure_to_felt() {
                let deployed = deploy_contract();
                let map_key = StructuredKey { a: 420, b: NestedKey { c: 421 } };
                
                deployed.insert_structure_to_felt(map_key, 123);
                
                let loaded = load(
                    deployed.contract_address, 
                    map_entry_address(selector!("structure_to_felt"), map_key.into()), 
                    1
                );
                assert(loaded == array![123].span(), 'wrong felt stored');
            }
        "#
        ),
        Contract::from_code_path(
            "StorageTester".to_string(),
            Path::new("tests/data/contracts/storage_tester.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn store_load_felt_to_felt() {
    let test = test_utils::test_case!(
        indoc!(
            r#"
            use array::ArrayTrait;
            use result::ResultTrait;
            use starknet::ContractAddress;
            use snforge_std::{ declare, ContractClassTrait, store, load, map_entry_address };

            #[starknet::interface]
            trait IStorageTester<TContractState> {
                fn insert_felt_to_felt(ref self: TContractState, key: felt252, value: felt252);
                fn read_felt_to_felt(self: @TContractState, key: felt252) -> felt252;
            }
            
            fn deploy_contract() -> IStorageTesterDispatcher {
                let contract = declare('StorageTester');
                let contract_address = contract.deploy(@array![]).unwrap();
                IStorageTesterDispatcher { contract_address }
            }

            #[test]
            fn store_felt_to_felt() {
                let deployed = deploy_contract();
                store(
                    deployed.contract_address, 
                    map_entry_address(selector!("felt_to_felt"), array![420]), 
                    array![123].span()
                );
                
                let stored_felt = deployed.read_felt_to_felt(420);
                assert(stored_felt == 123, 'wrong stored_felt'); 
            }
            
            #[test]
            fn load_felt_to_felt() {
                let deployed = deploy_contract();
                deployed.insert_felt_to_felt(420, 123);
                
                let loaded = load(
                    deployed.contract_address, 
                    map_entry_address(selector!("felt_to_felt"), array![420]), 
                    1
                );
                assert(loaded == array![123].span(), 'wrong felt stored');
            }
        "#
        ),
        Contract::from_code_path(
            "StorageTester".to_string(),
            Path::new("tests/data/contracts/storage_tester.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}
static INTEGRATION_RPC_URL: &str = "http://188.34.188.184:9545/rpc/v0_6";
#[test]
fn fork_store_load() {
    let test = test_utils::test_case!(formatdoc!(
        r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use starknet::contract_address_const;
            use snforge_std::{{ BlockTag, BlockId, load, store }};

            #[starknet::interface]
            trait IHelloStarknet<TContractState> {{
                fn increase_balance(ref self: TContractState, amount: felt252);
                fn get_balance(self: @TContractState) -> felt252;
            }}

            #[test]
            #[fork(url: "{}", block_id: BlockId::Number(313388))]
            fn fork_simple_decorator() {{
                let dispatcher = IHelloStarknetDispatcher {{
                    contract_address: contract_address_const::<3216637956526895219277698311134811322769343974163380838558193911733621219342>()
                }};

                let balance = dispatcher.get_balance();
                assert(balance == 2, 'Balance should be 2');
                let result = load(dispatcher.contract_address, selector!("balance"), 1);
                assert(*result.at(0) == 2, 'Wrong balance loaded');
                
                store(dispatcher.contract_address, selector!("balance"), array![100].span());

                let balance = dispatcher.get_balance();
                assert(balance == 100, 'Balance should be 100');
            }}
        "#,
        INTEGRATION_RPC_URL
    ).as_str());

    let result = run_test_case(&test);

    assert_passed!(result);
}
