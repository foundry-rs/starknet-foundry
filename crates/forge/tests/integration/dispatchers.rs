use crate::integration::common::runner::Contract;
use crate::{assert_passed, test_case};
use camino::Utf8PathBuf;
use std::path::Path;
use std::string::ToString;

use crate::integration::common::corelib::{corelib_path, predeployed_contracts};
use forge::run;
use indoc::indoc;

#[test]
fn simple_call_and_invoke() {
    let test = test_case!(
        indoc!(
            r#"
        use array::ArrayTrait;
        use result::ResultTrait;
        use option::OptionTrait;
        use traits::TryInto;
        use starknet::ContractAddress;
        use starknet::Felt252TryIntoContractAddress;
        use cheatcodes::PreparedContract;
            
        #[starknet::interface]
        trait IHelloStarknet<TContractState> {
            fn increase_balance(ref self: TContractState, amount: felt252);
            fn get_balance(self: @TContractState) -> felt252;
            fn do_a_panic(self: @TContractState);
            fn do_a_panic_with(self: @TContractState, panic_data: Array<felt252>);
        }

        #[test]
        fn call_and_invoke() {
            let class_hash = declare('HelloStarknet');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @ArrayTrait::new() };
            let contract_address = deploy(prepared).unwrap();
            let dispatcher = IHelloStarknetDispatcher { contract_address };

            let balance = dispatcher.get_balance();
            assert(balance == 0, 'balance == 0');

            dispatcher.increase_balance(100);

            let balance = dispatcher.get_balance();
            assert(balance == 100, 'balance == 100');
        }
    "#
        ),
        Contract::from_code_path(
            "HelloStarknet".to_string(),
            Path::new("tests/data/contracts/hello_starknet.cairo"),
        )
        .unwrap()
    );

    let result = run(
        &test.path().unwrap(),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();

    assert_passed!(result);
}

#[test]
fn advanced_types() {
    let test = test_case!(
        indoc!(
            r#"
        use array::ArrayTrait;
        use result::ResultTrait;
        use option::OptionTrait;
        use traits::TryInto;
        use starknet::ContractAddress;
        use starknet::Felt252TryIntoContractAddress;
        use cheatcodes::PreparedContract;
            

        #[starknet::interface]
        trait IERC20<TContractState> {
            fn get_name(self: @TContractState) -> felt252;
            fn get_symbol(self: @TContractState) -> felt252;
            fn get_decimals(self: @TContractState) -> u8;
            fn get_total_supply(self: @TContractState) -> u256;
            fn balance_of(self: @TContractState, account: ContractAddress) -> u256;
            fn allowance(self: @TContractState, owner: ContractAddress, spender: ContractAddress) -> u256;
            fn transfer(ref self: TContractState, recipient: ContractAddress, amount: u256);
            fn transfer_from(
                ref self: TContractState, sender: ContractAddress, recipient: ContractAddress, amount: u256
            );
            fn approve(ref self: TContractState, spender: ContractAddress, amount: u256);
            fn increase_allowance(ref self: TContractState, spender: ContractAddress, added_value: u256);
            fn decrease_allowance(
                ref self: TContractState, spender: ContractAddress, subtracted_value: u256
            );
        }

        #[test]
        fn advanced_types() {
            let mut calldata = ArrayTrait::new();
            calldata.append('token');   // name
            calldata.append('TKN');     // symbol
            calldata.append(18);        // decimals
            calldata.append(1111);      // initial supply low
            calldata.append(0);         // initial supply high
            calldata.append(1234);      // recipient
        
            let class_hash = declare('ERC20');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @calldata };
            let contract_address = deploy(prepared).unwrap();
            let dispatcher = IERC20Dispatcher { contract_address };
            let user_address: ContractAddress = 1234.try_into().unwrap();
            let other_user_address: ContractAddress = 9999.try_into().unwrap();
        
            let balance = dispatcher.balance_of(user_address);
            assert(balance == 1111_u256, 'balance == 1111');

            start_prank(contract_address, user_address);
            dispatcher.transfer(other_user_address, 1000_u256);
        
            let balance = dispatcher.balance_of(user_address);
            assert(balance == 111_u256, 'balance == 111');
            let balance = dispatcher.balance_of(other_user_address);
            assert(balance == 1000_u256, 'balance == 1000');
        }
    "#
        ),
        Contract::from_code_path(
            "ERC20".to_string(),
            Path::new("tests/data/contracts/erc20.cairo"),
        )
        .unwrap()
    );

    let result = run(
        &test.path().unwrap(),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();

    assert_passed!(result);
}

#[test]
fn handling_errors() {
    let test = test_case!(
        indoc!(
            r#"
        use array::ArrayTrait;
        use result::ResultTrait;
        use option::OptionTrait;
        use traits::TryInto;
        use starknet::ContractAddress;
        use starknet::Felt252TryIntoContractAddress;
        use cheatcodes::PreparedContract;
            
        #[starknet::interface]
        trait IHelloStarknet<TContractState> {
            fn increase_balance(ref self: TContractState, amount: felt252);
            fn get_balance(self: @TContractState) -> felt252;
            fn do_a_panic(self: @TContractState);
            fn do_a_panic_with(self: @TContractState, panic_data: Array<felt252>);
        }

        #[test]
        fn handling_errors() {
            let class_hash = declare('HelloStarknet');
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @ArrayTrait::new() };
            let contract_address = deploy(prepared).unwrap();
            let safe_dispatcher = IHelloStarknetSafeDispatcher { contract_address };
        
        
            match safe_dispatcher.do_a_panic() {
                Result::Ok(_) => panic_with_felt252('shouldve panicked'),
                Result::Err(panic_data) => {
                    assert(*panic_data.at(0) == 'PANIC', *panic_data.at(0));
                    assert(*panic_data.at(1) == 'DAYTAH', *panic_data.at(1));
                }
            };
        
            let mut panic_data = ArrayTrait::new();
            panic_data.append('capybara');
            match safe_dispatcher.do_a_panic_with(panic_data) {
                Result::Ok(_) => panic_with_felt252('shouldve panicked'),
                Result::Err(panic_data) => {
                    assert(panic_data.len() == 1, 'Wrong panic_data len');
                    assert(*panic_data.at(0) == 'capybara', *panic_data.at(0));
                }
            };
        
            match safe_dispatcher.do_a_panic_with(ArrayTrait::new()) {
                Result::Ok(_) => panic_with_felt252('shouldve panicked'),
                Result::Err(panic_data) => {
                    assert(panic_data.len() == 0, 'Non-empty panic_data');
                }
            };
        }
    "#
        ),
        Contract::from_code_path(
            "HelloStarknet".to_string(),
            Path::new("tests/data/contracts/hello_starknet.cairo"),
        )
        .unwrap()
    );

    let result = run(
        &test.path().unwrap(),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();

    assert_passed!(result);
}

#[test]
fn serding() {
    let test = test_case!(
        indoc!(
            r#"
        use array::ArrayTrait;
        use result::ResultTrait;
        use option::OptionTrait;
        use traits::TryInto;
        use starknet::ContractAddress;
        use starknet::Felt252TryIntoContractAddress;
        
        #[derive(Drop, Serde)]
        struct NestedStruct {
            d: felt252,
        }
        
        #[derive(Drop, Serde)]
        struct CustomStruct {
            a: felt252,
            b: felt252,
            c: NestedStruct,
        }
        
        #[derive(Drop, Serde)]
        struct AnotherCustomStruct {
            e: felt252,
        }
        
        #[starknet::interface]
        trait ISerding<T> {
            fn add_multiple_parts(
                self: @T,
                custom_struct: CustomStruct,
                another_struct: AnotherCustomStruct,
                standalone_arg: felt252
            ) -> felt252;
        }
        
        #[test]
        fn test_serding() {
            let class_hash = declare('Serding');
            let prepared = PreparedContract {
                class_hash, constructor_calldata: @ArrayTrait::new()
            };
            let contract_address = deploy(prepared).unwrap();
        
            let dispatcher = ISerdingDispatcher {
                contract_address
            };
        
            let ns = NestedStruct { d: 1 };
            let cs = CustomStruct { a: 2, b: 3, c: ns };
            let acs = AnotherCustomStruct { e: 4 };
            let standalone_arg = 5;
        
            let result = dispatcher.add_multiple_parts(cs, acs, standalone_arg);
        
            assert(result == 1 + 2 + 3 + 4 + 5, 'Invalid sum');
        }
    "#
        ),
        Contract::from_code_path(
            "Serding".to_string(),
            Path::new("tests/data/contracts/serding.cairo"),
        )
        .unwrap()
    );

    let result = run(
        &test.path().unwrap(),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();

    assert_passed!(result);
}

#[test]
#[allow(clippy::too_many_lines)]
fn proxy_storage() {
    let test = test_case!(
        indoc!(
            r#"
        use array::ArrayTrait;
        use result::ResultTrait;
        use option::OptionTrait;
        use traits::TryInto;
        use starknet::ContractAddress;
        use starknet::Felt252TryIntoContractAddress;
        
        #[derive(Drop, Serde, PartialEq, Copy)]
        struct NestedStruct {
            d: felt252,
        }
        
        #[derive(Drop, Serde, PartialEq, Copy)]
        struct CustomStruct {
            a: felt252,
            b: felt252,
            c: NestedStruct,
        }
        
        fn deploy_contract(name: felt252) -> ContractAddress {
            let class_hash = declare(name);
            let prepared = PreparedContract { class_hash, constructor_calldata: @ArrayTrait::new() };
            deploy(prepared).unwrap()
        }
        
        
        #[starknet::interface]
        trait ICaller<T> {
            fn call_executor(
                self: @T, executor_address: starknet::ContractAddress, custom_struct: CustomStruct
            ) -> felt252;
        }
        
        #[starknet::interface]
        trait IExecutor<T> {
            fn read_storage(ref self: T) -> CustomStruct;
        }
        
        #[test]
        fn test_proxy_storage() {
            let caller_address = deploy_contract('Caller');
            let executor_address = deploy_contract('Executor');
        
            let caller_dispatcher = ICallerDispatcher { contract_address: caller_address };
            let executor_dispatcher = IExecutorDispatcher { contract_address: executor_address };
        
            let ns = NestedStruct { d: 6 };
            let cs = CustomStruct { a: 2, b: 3, c: ns };
        
            let result = caller_dispatcher.call_executor(executor_address, cs);
        
            assert(result == 6 + 5, 'Invalid result');
        
            let storage_after = executor_dispatcher.read_storage();
        
            assert(storage_after == cs, 'Invalid storage');
        }
    "#
        ),
        Contract::new(
            "Caller",
            indoc!(
                r#"
            #[derive(Drop, Serde, starknet::Store)]
            struct NestedStruct {
                d: felt252, 
            }
            
            #[derive(Drop, Serde, starknet::Store)]
            struct CustomStruct {
                a: felt252,
                b: felt252,
                c: NestedStruct,
            }
            
            #[starknet::contract]
            mod Caller {
                use super::CustomStruct;
                use result::ResultTrait;
            
                #[starknet::interface]
                trait IExecutor<T> {
                    fn store_and_add_5(self: @T, custom_struct: CustomStruct) -> felt252;
                }
            
                #[storage]
                struct Storage {}
            
                #[external(v0)]
                fn call_executor(
                    self: @ContractState,
                    executor_address: starknet::ContractAddress,
                    custom_struct: CustomStruct
                ) -> felt252 {
                    let safe_dispatcher = IExecutorDispatcher { contract_address: executor_address };
                    safe_dispatcher.store_and_add_5(custom_struct)
                }
            }
            "#
            )
        ),
        Contract::new(
            "Executor",
            indoc!(
                r#"
            #[derive(Drop, Serde, starknet::Store)]
            struct NestedStruct {
                d: felt252, 
            }
            
            #[derive(Drop, Serde, starknet::Store)]
            struct CustomStruct {
                a: felt252,
                b: felt252,
                c: NestedStruct,
            }
            
            #[starknet::contract]
            mod Executor {
                use super::CustomStruct;
                #[storage]
                struct Storage {
                    thing: CustomStruct
                }
            
                #[external(v0)]
                fn store_and_add_5(ref self: ContractState, custom_struct: CustomStruct) -> felt252 {
                    self.thing.write(custom_struct);
                    5 + self.thing.read().c.d
                }
            
                #[external(v0)]
                fn read_storage(ref self: ContractState) -> CustomStruct {
                    self.thing.read()
                }
            }
            "#
            )
        )
    );

    let result = run(
        &test.path().unwrap(),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();

    assert_passed!(result);
}
