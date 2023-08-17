use crate::integration::common::corelib::{corelib_path, predeployed_contracts};
use crate::integration::common::runner::Contract;
use crate::{assert_case_output_contains, assert_failed, assert_passed, test_case};
use camino::Utf8PathBuf;
use forge::run;
use indoc::indoc;

#[test]
#[allow(clippy::too_many_lines)]
fn library_call_syscall() {
    let test = test_case!(
        indoc!(
            r#"
        use array::ArrayTrait;
        use result::ResultTrait;
        use option::OptionTrait;
        use traits::TryInto;
        use starknet::ContractAddress;
        use starknet::Felt252TryIntoContractAddress;
        use starknet::ClassHash;
        use snforge_std::{ declare, PreparedContract, deploy };

        #[starknet::interface]
        trait ICaller<TContractState> {
            fn call_add_two(
                self: @TContractState, class_hash: ClassHash, number: felt252
            ) -> felt252;
        }

        #[starknet::interface]
        trait IExecutor<TContractState> {
            fn add_two(ref self: TContractState, number: felt252) -> felt252;
            fn get_thing(self: @TContractState) -> felt252;
        }

        fn deploy_contract(name: felt252) -> ContractAddress {
            let class_hash = declare(name);
            let prepared = PreparedContract {
                class_hash, constructor_calldata: @ArrayTrait::new()
            };
            deploy(prepared).unwrap()
        }

        #[test]
        fn test_library_call() {
            let caller_address = deploy_contract('Caller');
            let caller_safe_dispatcher = ICallerSafeDispatcher {
                contract_address: caller_address
            };

            let executor_class_hash = declare('Executor');
            let prepared = PreparedContract {
                class_hash: executor_class_hash, constructor_calldata: @ArrayTrait::new()
            };

            let executor_address = deploy(prepared).unwrap();
            let executor_safe_dispatcher = IExecutorSafeDispatcher {
                contract_address: executor_address
            };

            let thing = executor_safe_dispatcher.get_thing().unwrap();
            assert(thing == 5, 'invalid thing');

            let result = caller_safe_dispatcher.call_add_two(executor_class_hash, 420).unwrap();
            assert(result == 422, 'invalid result');

            let thing = executor_safe_dispatcher.get_thing().unwrap();
            assert(thing == 5, 'invalid thing');
        }
        "#
        ),
        Contract::new(
            "Caller",
            indoc!(
                r#"
                #[starknet::contract]
                mod Caller {
                    use result::ResultTrait;
                    use starknet::ClassHash;
                    use starknet::library_call_syscall;

                    #[starknet::interface]
                    trait IExecutor<TContractState> {
                        fn add_two(ref self: ContractState, number: felt252) -> felt252;
                    }

                    #[storage]
                    struct Storage {}

                    #[external(v0)]
                    fn call_add_two(
                        self: @ContractState, class_hash: ClassHash, number: felt252
                    ) -> felt252 {
                        let safe_lib_dispatcher = IExecutorSafeLibraryDispatcher { class_hash };
                        safe_lib_dispatcher.add_two(number).unwrap()
                    }
                }
                "#
            )
        ),
        Contract::new(
            "Executor",
            indoc!(
                r#"
                #[starknet::contract]
                mod Executor {
                    #[storage]
                    struct Storage {
                        thing: felt252
                    }

                    #[constructor]
                    fn constructor(ref self: ContractState) {
                        assert(self.thing.read() == 0, 'default value should be 0');
                        self.thing.write(5);
                    }

                    #[external(v0)]
                    fn add_two(ref self: ContractState, number: felt252) -> felt252 {
                        self.thing.write(10);
                        number + 2
                    }

                    #[external(v0)]
                    fn get_thing(self: @ContractState) -> felt252 {
                        self.thing.read()
                    }
                }
                "#
            )
        )
    );
    let result = run(
        &test.path().unwrap(),
        &String::from("src"),
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
fn test_call_syscall_fail_in_test_fn() {
    let test = test_case!(indoc!(
        r#"
        use starknet::{ get_block_timestamp };
        #[test]
        fn test_execute_disallowed_syscall() {
            get_block_timestamp();
        }
    "#
    ));

    let result = run(
        &test.path().unwrap(),
        &String::from("src"),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();

    assert_case_output_contains!(
        result,
        "test_execute_disallowed_syscall",
        "starknet syscalls cannot be used in tests"
    );
    assert_failed!(result);
}

#[test]
fn test_keccak_syscall() {
    let test = test_case!(indoc!(
        r#"
        use array::ArrayTrait;
        use keccak::cairo_keccak;

        #[test]
        fn test_execute_cairo_keccak() {
            let mut input = array![
                0x0000000000000001,
                0x0000000000000002,
                0x0000000000000003,
                0x0000000000000004,
                0x0000000000000005,
                0x0000000000000006,
                0x0000000000000007,
                0x0000000000000008,
                0x0000000000000009,
                0x000000000000000a,
                0x000000000000000b,
                0x000000000000000c,
                0x000000000000000d,
                0x000000000000000e,
                0x000000000000000f,
                0x0000000000000010,
                0x0000000000000011
            ];

            let res = keccak::cairo_keccak(ref input, 0, 0);

            assert(@res.low == @0x5d291eebae35b254ff50ec1fc57832e8, 'Wrong hash low');
            assert(@res.high == @0x210740d45b1fe2ac908a497ef45509f5, 'Wrong hash high');
        }
    "#
    ));

    let result = run(
        &test.path().unwrap(),
        &String::from("src"),
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
