use indoc::indoc;
use std::path::Path;
use test_utils::runner::{Contract, assert_case_output_contains, assert_failed, assert_passed};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
#[allow(clippy::too_many_lines)]
fn library_call_syscall_is_usable() {
    let test = test_case!(
        indoc!(
            r#"
        use core::clone::Clone;
        use starknet::ContractAddress;
        use starknet::ClassHash;
        use snforge_std::{ declare, DeclareResultTrait, ContractClassTrait };

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

        fn deploy_contract(name: ByteArray) -> ContractAddress {
            let contract = declare(name).unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
            contract_address
        }

        #[test]
        fn library_call_syscall_is_usable() {
            let caller_address = deploy_contract("Caller");
            let caller_safe_dispatcher = ICallerDispatcher {
                contract_address: caller_address
            };

            let executor_contract = declare("Executor").unwrap().contract_class();
            let executor_class_hash = executor_contract.class_hash.clone();

            let (executor_address, _) = executor_contract.deploy(@ArrayTrait::new()).unwrap();
            let executor_safe_dispatcher = IExecutorDispatcher {
                contract_address: executor_address
            };
            let thing = executor_safe_dispatcher.get_thing();
            assert(thing == 5, 'invalid thing');

            let result = caller_safe_dispatcher.call_add_two(executor_class_hash, 420);
            assert(result == 422, 'invalid result');

            let thing = executor_safe_dispatcher.get_thing();
            assert(thing == 5, 'invalid thing');
        }
        "#
        ),
        Contract::new(
            "Caller",
            indoc!(
                r"
                use starknet::ClassHash;

                #[starknet::interface]
                trait ICaller<TContractState> {
                    fn call_add_two(
                        self: @TContractState, class_hash: ClassHash, number: felt252
                    ) -> felt252;
                }

                #[starknet::contract]
                mod Caller {
                    use result::ResultTrait;
                    use starknet::ClassHash;
                    use starknet::library_call_syscall;

                    #[starknet::interface]
                    trait IExecutor<TContractState> {
                        fn add_two(ref self: TContractState, number: felt252) -> felt252;
                    }

                    #[storage]
                    struct Storage {}

                    #[abi(embed_v0)]
                    impl CallerImpl of super::ICaller<ContractState> {
                        fn call_add_two(
                            self: @ContractState, class_hash: ClassHash, number: felt252
                        ) -> felt252 {
                            let safe_lib_dispatcher = IExecutorLibraryDispatcher { class_hash };
                            safe_lib_dispatcher.add_two(number)
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
                    fn add_two(ref self: TContractState, number: felt252) -> felt252;
                    fn get_thing(self: @TContractState) -> felt252;
                }

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

                    #[abi(embed_v0)]
                    impl ExecutorImpl of super::IExecutor<ContractState> {
                        fn add_two(ref self: ContractState, number: felt252) -> felt252 {
                            self.thing.write(10);
                            number + 2
                        }

                        fn get_thing(self: @ContractState) -> felt252 {
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
fn keccak_syscall_is_usable() {
    let test = test_case!(indoc!(
        r"
        use array::ArrayTrait;
        use starknet::syscalls::keccak_syscall;
        use starknet::SyscallResultTrait;

        #[test]
        fn keccak_syscall_is_usable() {
            let input = array![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17];
            assert(
                @keccak_syscall(input.span()).unwrap_syscall()
                == @u256 { low: 0xec687be9c50d2218388da73622e8fdd5, high: 0xd2eb808dfba4703c528d145dfe6571af },
                'Wrong hash value'
            );
        }
    "
    ));

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn keccak_syscall_too_small_input() {
    let test = test_case!(indoc!(
        r"
        use array::ArrayTrait;
        use starknet::syscalls::keccak_syscall;
        use starknet::SyscallResultTrait;

        #[test]
        fn keccak_syscall_too_small_input() {
            let input = array![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
            assert(
                @keccak_syscall(input.span()).unwrap_syscall()
                == @u256 { low: 0xec687be9c50d2218388da73622e8fdd5, high: 0xd2eb808dfba4703c528d145dfe6571af },
                'Wrong hash value'
            );
        }
    "
    ));

    let result = run_test_case(&test);

    assert_case_output_contains(
        &result,
        "keccak_syscall_too_small_input",
        "Invalid input length",
    );

    assert_failed(&result);
}

#[test]
fn cairo_keccak_is_usable() {
    let test = test_case!(indoc!(
        r"
        use array::ArrayTrait;
        use keccak::cairo_keccak;

        #[test]
        fn cairo_keccak_is_usable() {
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
    "
    ));

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn keccak_syscall_in_contract() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use snforge_std::{ declare, DeclareResultTrait, ContractClassTrait };

            #[starknet::interface]
            trait IHelloKeccak<TContractState> {
                fn run_keccak(ref self: TContractState, input: Array<u64>) -> u256;
            }

            #[test]
            fn keccak_syscall_in_contract() {
                let contract = declare("HelloKeccak").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IHelloKeccakDispatcher { contract_address };

                let res = dispatcher.run_keccak(array![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17]);
                assert(
                    res == u256 { low: 0xec687be9c50d2218388da73622e8fdd5, high: 0xd2eb808dfba4703c528d145dfe6571af },
                    'Wrong hash value'
                );
            }
        "#
        ),
        Contract::from_code_path(
            "HelloKeccak".to_string(),
            Path::new("tests/data/contracts/keccak_usage.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn compare_keccak_from_contract_with_plain_keccak() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::syscalls::keccak_syscall;
            use starknet::SyscallResultTrait;
            use snforge_std::{ declare, DeclareResultTrait, ContractClassTrait };

            #[starknet::interface]
            trait IHelloKeccak<TContractState> {
                fn run_keccak(ref self: TContractState, input: Array<u64>) -> u256;
            }

            #[test]
            fn compare_keccak_from_contract_with_plain_keccak() {
                let contract = declare("HelloKeccak").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IHelloKeccakDispatcher { contract_address };

                let input = array![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17];
                let keccak = keccak_syscall(input.span()).unwrap_syscall();
                let contract_keccak = dispatcher.run_keccak(input);

                assert(contract_keccak == keccak, 'Keccaks dont match');
            }
        "#
        ),
        Contract::from_code_path(
            "HelloKeccak".to_string(),
            Path::new("tests/data/contracts/keccak_usage.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}
