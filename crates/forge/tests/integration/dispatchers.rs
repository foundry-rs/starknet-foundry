use indoc::indoc;
use std::path::Path;
use test_utils::runner::{assert_case_output_contains, assert_failed, assert_passed, Contract};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

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
        use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };

        #[starknet::interface]
        trait IHelloStarknet<TContractState> {
            fn increase_balance(ref self: TContractState, amount: felt252);
            fn get_balance(self: @TContractState) -> felt252;
            fn do_a_panic(self: @TContractState);
            fn do_a_panic_with(self: @TContractState, panic_data: Array<felt252>);
        }

        #[test]
        fn simple_call_and_invoke() {
            let contract = declare("HelloStarknet").unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
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

    let result = run_test_case(&test);

    assert_passed(&result);
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
        use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait, start_cheat_caller_address };

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

            let contract = declare("ERC20").unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@calldata).unwrap();
            let dispatcher = IERC20Dispatcher { contract_address };
            let user_address: ContractAddress = 1234.try_into().unwrap();
            let other_user_address: ContractAddress = 9999.try_into().unwrap();

            let balance = dispatcher.balance_of(user_address);
            assert(balance == 1111_u256, 'balance == 1111');

            start_cheat_caller_address(contract_address, user_address);
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

    let result = run_test_case(&test);

    assert_passed(&result);
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
        use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };
        use starknet::contract_address_const;

        #[starknet::interface]
        trait IHelloStarknet<TContractState> {
            fn increase_balance(ref self: TContractState, amount: felt252);
            fn get_balance(self: @TContractState) -> felt252;
            fn do_a_panic(self: @TContractState);
            fn do_a_panic_with(self: @TContractState, panic_data: Array<felt252>);
        }

        #[test]
        #[feature("safe_dispatcher")]
        fn handling_execution_errors() {
            let contract = declare("HelloStarknet").unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
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

        #[test]
        #[feature("safe_dispatcher")]
        fn handling_missing_contract_error() {
            let safe_dispatcher = IHelloStarknetSafeDispatcher {
                contract_address: contract_address_const::<371937219379012>()
            };

            match safe_dispatcher.do_a_panic() {
                Result::Ok(_) => panic_with_felt252('shouldve panicked'),
                Result::Err(_) => {
                    // Would be nice to assert the error here once it is possible in cairo
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

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn handling_bytearray_based_errors() {
    let test = test_case!(
        indoc!(
            r#"
        use starknet::ContractAddress;
        use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };
        use snforge_std::byte_array::try_deserialize_bytearray_error;
        use core::byte_array::BYTE_ARRAY_MAGIC;

        #[starknet::interface]
        trait IHelloStarknet<TContractState> {
            fn do_a_panic_with_bytearray(self: @TContractState);
            fn do_a_panic_with(self: @TContractState, args: Array<felt252>);
        }

        #[test]
        #[feature("safe_dispatcher")]
        fn handling_errors() {
            let contract = declare("HelloStarknet").unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
            let safe_dispatcher = IHelloStarknetSafeDispatcher { contract_address };

            let panic_data = safe_dispatcher.do_a_panic_with_bytearray().unwrap_err();
            let str_err = try_deserialize_bytearray_error(panic_data.span()).expect('wrong format');
            assert(
                str_err == "This is a very long\n and multiline message that is certain to fill the buffer",
                'wrong string received'
            );

           // Not a bytearray
           let panic_data = safe_dispatcher.do_a_panic_with(array![123, 321]).unwrap_err();
           try_deserialize_bytearray_error(panic_data.span()).expect_err('Parsing unexpectedy succeeded');

           // Malformed bytearray
           let panic_data = safe_dispatcher.do_a_panic_with(array![BYTE_ARRAY_MAGIC, 321]).unwrap_err();
           try_deserialize_bytearray_error(panic_data.span()).expect_err('Parsing unexpectedy succeeded');
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

    assert_passed(&result);
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
        use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };

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
        fn serding() {
            let contract = declare("Serding").unwrap().contract_class();
            let (contract_address, _) = contract.deploy( @ArrayTrait::new()).unwrap();

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

    let result = run_test_case(&test);

    assert_passed(&result);
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
        use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };


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

        fn deploy_contract(name: ByteArray) -> ContractAddress {
            let contract = declare(name).unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
            contract_address
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
        fn proxy_storage() {
            let caller_address = deploy_contract("Caller");
            let executor_address = deploy_contract("Executor");

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
                r"
            use starknet::ContractAddress;

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

            #[starknet::interface]
            trait ICaller<TContractState> {
                fn call_executor(
                    self: @TContractState,
                    executor_address: ContractAddress,
                    custom_struct: CustomStruct
                ) -> felt252;
            }

            #[starknet::contract]
            mod Caller {
                use super::CustomStruct;
                use result::ResultTrait;
                use starknet::ContractAddress;

                #[starknet::interface]
                trait IExecutor<T> {
                    fn store_and_add_5(self: @T, custom_struct: CustomStruct) -> felt252;
                }

                #[storage]
                struct Storage {}

                #[abi(embed_v0)]
                impl CallerImpl of super::ICaller<ContractState> {
                    fn call_executor(
                        self: @ContractState,
                        executor_address: ContractAddress,
                        custom_struct: CustomStruct
                    ) -> felt252 {
                        let safe_dispatcher = IExecutorDispatcher { contract_address: executor_address };
                        safe_dispatcher.store_and_add_5(custom_struct)
                    }
                }
            }
            "
            )
        ),
        Contract::new(
            "Executor",
            indoc!(
                r"
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

            #[starknet::interface]
            trait IExecutor<TContractState> {
                fn store_and_add_5(ref self: TContractState, custom_struct: CustomStruct) -> felt252;
                fn read_storage(ref self: TContractState) -> CustomStruct;
            }

            #[starknet::contract]
            mod Executor {
                use super::CustomStruct;
                #[storage]
                struct Storage {
                    thing: CustomStruct
                }

                #[abi(embed_v0)]
                impl ExecutorImpl of super::IExecutor<ContractState> {
                    fn store_and_add_5(ref self: ContractState, custom_struct: CustomStruct) -> felt252 {
                        self.thing.write(custom_struct);
                        5 + self.thing.read().c.d
                    }

                    fn read_storage(ref self: ContractState) -> CustomStruct {
                        self.thing.read()
                    }
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
#[allow(clippy::too_many_lines)]
#[ignore] // Not doable right now in production
fn proxy_dispatcher_panic() {
    let test = test_case!(
        indoc!(
            r#"
        use array::ArrayTrait;
        use result::ResultTrait;
        use option::OptionTrait;
        use traits::TryInto;
        use traits::Into;
        use starknet::ContractAddress;
        use starknet::Felt252TryIntoContractAddress;
        use snforge_std::{ declare, ContractClassTrait };

        fn deploy_contract(name: ByteArray, constructor_calldata: @Array<felt252>) -> ContractAddress {
            let contract = declare(name).unwrap();
            let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
            contract_address
        }

        #[starknet::interface]
        trait ICaller<T> {
            fn invoke_executor(ref self: T);
        }

        #[test]
        fn proxy_dispatcher_panic() {
            let executor_address = deploy_contract("Executor", @ArrayTrait::new());
            let caller_constructor_calldata: Array<felt252> = array![executor_address.into()];
            let caller_address = deploy_contract("Caller", @caller_constructor_calldata);

            let caller_dispatcher = ICallerSafeDispatcher { contract_address: caller_address };

            match caller_dispatcher.invoke_executor() {
                Result::Ok(_) => panic_with_felt252('should have panicked'),
                Result::Err(x) => assert(*x.at(0) == 'panic_msg', 'wrong panic msg')
            }
        }
    "#
        ),
        Contract::new(
            "Caller",
            indoc!(
                r"
            #[starknet::interface]
            trait ICaller<TContractState> {
                fn invoke_executor(
                    self: @TContractState,
                );
            }

            #[starknet::contract]
            mod Caller {
                use result::ResultTrait;
                use starknet::ContractAddress;

                #[starknet::interface]
                trait IExecutor<T> {
                    fn invoke_with_panic(self: @T);
                }

                #[storage]
                struct Storage {
                    executor_address: ContractAddress
                }

                #[constructor]
                fn constructor(ref self: ContractState, executor_address: ContractAddress) {
                    self.executor_address.write(executor_address);
                }

                #[abi(embed_v0)]
                impl CallerImpl of super::ICaller<ContractState> {
                    fn invoke_executor(
                        self: @ContractState,
                    ) {
                        let dispatcher = IExecutorDispatcher { contract_address: self.executor_address.read() };
                        dispatcher.invoke_with_panic()
                    }
                }
            }
            "
            )
        ),
        Contract::new(
            "Executor",
            indoc!(
                r"
            #[starknet::interface]
            trait IExecutor<TContractState> {
                fn invoke_with_panic(ref self: TContractState);
            }

            #[starknet::contract]
            mod Executor {
                #[storage]
                struct Storage {}

                #[abi(embed_v0)]
                impl ExecutorImpl of super::IExecutor<ContractState> {
                    fn invoke_with_panic(ref self: ContractState) {
                        panic_with_felt252('panic_msg');
                    }
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
fn nonexistent_method_call() {
    let test = test_case!(
        indoc!(
            r#"
        use array::ArrayTrait;
        use result::ResultTrait;
        use option::OptionTrait;
        use traits::TryInto;
        use traits::Into;
        use starknet::ContractAddress;
        use starknet::Felt252TryIntoContractAddress;
        use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };


        fn deploy_contract(name: ByteArray, constructor_calldata: @Array<felt252>) -> ContractAddress {
            let contract = declare(name).unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
            contract_address
        }

        #[starknet::interface]
        trait ICaller<T> {
            fn invoke_nonexistent(ref self: T);
        }

        #[test]
        fn nonexistent_method_call() {
            let contract_address = deploy_contract("Contract", @ArrayTrait::new());

            let caller_dispatcher = ICallerDispatcher { contract_address };
            caller_dispatcher.invoke_nonexistent();
        }
    "#
        ),
        Contract::new(
            "Contract",
            indoc!(
                r"
            #[starknet::contract]
            mod Contract {
                #[storage]
                struct Storage {
                }
            }
            "
            )
        )
    );

    let result = run_test_case(&test);

    assert_failed(&result);
    assert_case_output_contains(
       & result,
        "nonexistent_method_call",
        "Entry point selector 0x1fdb214e1495025fa4baf660d34f03c0d8b5037cf10311d2a3202a806aa9485 not found in contract"
    );
}

#[test]
fn nonexistent_libcall_function() {
    let test = test_case!(
        indoc!(
            r#"
        use core::clone::Clone;
        use array::ArrayTrait;
        use result::ResultTrait;
        use option::OptionTrait;
        use traits::TryInto;
        use traits::Into;
        use starknet::ContractAddress;
        use starknet::Felt252TryIntoContractAddress;
        use starknet::ClassHash;
        use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };

        fn deploy_contract(name: ByteArray) -> ContractAddress {
            let contract = declare(name).unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
            contract_address
        }

        #[starknet::interface]
        trait IContract<T> {
            fn invoke_nonexistent_libcall_from_contract(ref self: T, class_hash: ClassHash);
        }

        #[test]
        fn nonexistent_libcall_function() {
            let class = declare("Contract").unwrap().contract_class().clone();
            let contract_address = deploy_contract("LibCaller");

            let dispatcher = IContractDispatcher { contract_address };
            dispatcher.invoke_nonexistent_libcall_from_contract(class.class_hash);
        }
        "#
        ),
        Contract::new(
            "LibCaller",
            indoc!(
                r"
                use starknet::ClassHash;

                #[starknet::interface]
                trait IContract<TContractState> {
                    fn invoke_nonexistent_libcall_from_contract(ref self: TContractState, class_hash: ClassHash);
                }

                #[starknet::contract]
                mod LibCaller {
                    use starknet::ClassHash;
                    use result::ResultTrait;
                    use array::ArrayTrait;

                    #[storage]
                    struct Storage {}

                    #[starknet::interface]
                    trait ICaller<T> {
                        fn invoke_nonexistent(ref self: T);
                    }

                    #[abi(embed_v0)]
                    impl ContractImpl of super::IContract<ContractState> {
                        fn invoke_nonexistent_libcall_from_contract(ref self: ContractState, class_hash: ClassHash) {
                            let lib_dispatcher = ICallerLibraryDispatcher { class_hash };

                            lib_dispatcher.invoke_nonexistent();
                        }
                    }
                }
                "
            )
        ),
        Contract::new(
            "Contract",
            indoc!(
                r"
            #[starknet::contract]
            mod Contract {
                #[storage]
                struct Storage {
                }
            }
            "
            )
        )
    );

    let result = run_test_case(&test);

    assert_failed(&result);

    assert_case_output_contains(
        &result,
        "nonexistent_libcall_function",
        "(0x454e545259504f494e545f4e4f545f464f554e44 ('ENTRYPOINT_NOT_FOUND'), 0x454e545259504f494e545f4641494c4544 ('ENTRYPOINT_FAILED'))"
    );
}

#[test]
fn undeclared_class_call() {
    let test = test_case!(indoc!(
        r"
        use starknet::ContractAddress;
        use traits::TryInto;
        use option::OptionTrait;

        #[starknet::interface]
        trait IContract<T> {
            fn invoke_nonexistent(ref self: T);
        }

        #[test]
        fn undeclared_class_call() {
            let dispatcher = IContractDispatcher { contract_address: 5.try_into().unwrap() };
            dispatcher.invoke_nonexistent();
        }
        "
    ));

    let result = run_test_case(&test);

    assert_failed(&result);
    assert_case_output_contains(
        &result,
        "undeclared_class_call",
        "Contract not deployed at address: 0x5",
    );
}

#[test]
fn nonexistent_class_libcall() {
    let test = test_case!(
        indoc!(
            r#"
        use array::ArrayTrait;
        use result::ResultTrait;
        use option::OptionTrait;
        use starknet::ContractAddress;
        use starknet::ClassHash;
        use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait };

        fn deploy_contract(name: ByteArray) -> ContractAddress {
            let contract = declare(name).unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
            contract_address
        }

        #[starknet::interface]
        trait IContract<T> {
            fn invoke_nonexistent_libcall_from_contract(ref self: T);
        }

        #[test]
        fn test_nonexistent_libcall() {
            let contract_address = deploy_contract("LibCaller");
            let dispatcher = IContractDispatcher { contract_address };
            dispatcher.invoke_nonexistent_libcall_from_contract();
        }
        "#
        ),
        Contract::new(
            "LibCaller",
            indoc!(
                r"
                #[starknet::interface]
                trait IContract<TContractState> {
                    fn invoke_nonexistent_libcall_from_contract(ref self: TContractState);
                }

                #[starknet::contract]
                mod LibCaller {
                    use starknet::class_hash::class_hash_try_from_felt252;
                    use starknet::ClassHash;
                    use result::ResultTrait;
                    use array::ArrayTrait;
                    use traits::TryInto;
                    use option::OptionTrait;

                    #[storage]
                    struct Storage {}

                    #[starknet::interface]
                    trait ICaller<T> {
                        fn invoke_nonexistent(ref self: T);
                    }

                    #[abi(embed_v0)]
                    impl ContractImpl of super::IContract<ContractState> {
                        fn invoke_nonexistent_libcall_from_contract(ref self: ContractState) {
                            let target_class_hash: ClassHash = class_hash_try_from_felt252(5_felt252).unwrap();
                            let lib_dispatcher = ICallerLibraryDispatcher { class_hash: target_class_hash  };
                            lib_dispatcher.invoke_nonexistent();
                        }
                    }
                }
                "
            )
        ),
        Contract::new(
            "Contract",
            indoc!(
                r"
            #[starknet::contract]
            mod Contract {
                #[storage]
                struct Storage {
                }
            }
            "
            )
        )
    );

    let result = run_test_case(&test);

    assert_failed(&result);
    assert_case_output_contains(&result, "test_nonexistent_libcall", "Class with hash");
    assert_case_output_contains(&result, "test_nonexistent_libcall", "is not declared.");
}
