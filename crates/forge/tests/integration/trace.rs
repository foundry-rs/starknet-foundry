use indoc::indoc;
use std::path::Path;
use test_utils::runner::{assert_passed, Contract};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
#[expect(clippy::too_many_lines)]
fn trace_deploy() {
    let test = test_case!(
        indoc!(
            r#"
            use core::clone::Clone;
            use snforge_std::{declare, ContractClassTrait, DeclareResultTrait, test_address, test_selector};
            use snforge_std::trace::{CallEntryPoint, CallType, EntryPointType, get_call_trace, CallTrace, CallResult};

            use starknet::{SyscallResultTrait, deploy_syscall, ContractAddress};

            #[test]
            fn test_deploy_trace_info() {
                let proxy = declare("TraceInfoProxy").unwrap().contract_class().clone();
                let checker = declare("TraceInfoChecker").unwrap().contract_class();

                let (checker_address, _) = checker.deploy(@array![]).unwrap();

                let (proxy_address1, _) = proxy.deploy(@array![checker_address.into()]).unwrap();

                let (proxy_address2, _) = deploy_syscall(
                    proxy.class_hash, 0, array![checker_address.into()].span(), false
                )
                    .unwrap_syscall();

                let (proxy_address_3, _) = proxy
                    .deploy_at(@array![checker_address.into()], 123.try_into().unwrap())
                    .unwrap();

                assert_trace(
                    get_call_trace(), proxy_address1, proxy_address2, proxy_address_3, checker_address
                );
            }

            fn assert_trace(
                trace: CallTrace,
                proxy_address1: ContractAddress,
                proxy_address2: ContractAddress,
                proxy_address3: ContractAddress,
                checker_address: ContractAddress
            ) {
                let expected_trace = CallTrace {
                    entry_point: CallEntryPoint {
                        entry_point_type: EntryPointType::External,
                        entry_point_selector: test_selector(),
                        calldata: array![],
                        contract_address: test_address(),
                        caller_address: 0.try_into().unwrap(),
                        call_type: CallType::Call,
                    },
                    nested_calls: array![
                        CallTrace {
                            entry_point: CallEntryPoint {
                                entry_point_type: EntryPointType::Constructor,
                                entry_point_selector: selector!("constructor"),
                                calldata: array![checker_address.into()],
                                contract_address: proxy_address1,
                                caller_address: test_address(),
                                call_type: CallType::Call,
                            },
                            nested_calls: array![
                                CallTrace {
                                    entry_point: CallEntryPoint {
                                        entry_point_type: EntryPointType::External,
                                        entry_point_selector: selector!("from_proxy"),
                                        calldata: array![1],
                                        contract_address: checker_address,
                                        caller_address: proxy_address1,
                                        call_type: CallType::Call,
                                    },
                                    nested_calls: array![],
                                    result: CallResult::Success(array![101])
                                }
                            ],
                            result: CallResult::Success(array![])
                        },
                        CallTrace {
                            entry_point: CallEntryPoint {
                                entry_point_type: EntryPointType::Constructor,
                                entry_point_selector: selector!("constructor"),
                                calldata: array![checker_address.into()],
                                contract_address: proxy_address2,
                                caller_address: test_address(),
                                call_type: CallType::Call,
                            },
                            nested_calls: array![
                                CallTrace {
                                    entry_point: CallEntryPoint {
                                        entry_point_type: EntryPointType::External,
                                        entry_point_selector: selector!("from_proxy"),
                                        calldata: array![1],
                                        contract_address: checker_address,
                                        caller_address: proxy_address2,
                                        call_type: CallType::Call,
                                    },
                                    nested_calls: array![],
                                    result: CallResult::Success(array![101])
                                }
                            ],
                            result: CallResult::Success(array![])
                        },
                        CallTrace {
                            entry_point: CallEntryPoint {
                                entry_point_type: EntryPointType::Constructor,
                                entry_point_selector: selector!("constructor"),
                                calldata: array![checker_address.into()],
                                contract_address: proxy_address3,
                                caller_address: test_address(),
                                call_type: CallType::Call,
                            },
                            nested_calls: array![
                                CallTrace {
                                    entry_point: CallEntryPoint {
                                        entry_point_type: EntryPointType::External,
                                        entry_point_selector: selector!("from_proxy"),
                                        calldata: array![1],
                                        contract_address: checker_address,
                                        caller_address: proxy_address3,
                                        call_type: CallType::Call,
                                    },
                                    nested_calls: array![],
                                    result: CallResult::Success(array![101])
                                }
                            ],
                            result: CallResult::Success(array![])
                        }
                    ],
                    result: CallResult::Success(array![])
                };

                assert(trace == expected_trace, '');
            }
        "#
        ),
        Contract::from_code_path(
            "TraceInfoProxy".to_string(),
            Path::new("tests/data/contracts/trace_info_proxy.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "TraceInfoChecker".to_string(),
            Path::new("tests/data/contracts/trace_info_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
#[expect(clippy::too_many_lines)]
fn trace_call() {
    let test = test_case!(
        indoc!(
            r#"
            use core::clone::Clone;
            use snforge_std::{declare, ContractClassTrait, DeclareResultTrait, test_address, test_selector, start_cheat_caller_address};
            use snforge_std::trace::{CallTrace, CallEntryPoint, CallType, EntryPointType, get_call_trace, CallResult};

            use starknet::{ContractAddress, ClassHash};

            #[starknet::interface]
            trait ITraceInfoProxy<T> {
                fn with_libcall(self: @T, class_hash: ClassHash) -> felt252;
                fn regular_call(self: @T, contract_address: ContractAddress) -> felt252;
                fn with_panic(self: @T, contract_address: ContractAddress);
                fn call_two(self: @T, checker_address: ContractAddress, dummy_address: ContractAddress);
            }

            #[starknet::interface]
            trait ITraceInfoChecker<T> {
                fn from_proxy(self: @T, data: felt252) -> felt252;
                fn panic(self: @T);
            }

            #[starknet::interface]
            trait ITraceDummy<T> {
                fn from_proxy(ref self: T);
            }

            #[test]
            fn test_call_trace_info() {
                let proxy = declare("TraceInfoProxy").unwrap().contract_class();
                let checker = declare("TraceInfoChecker").unwrap().contract_class().clone();
                let dummy = declare("TraceDummy").unwrap().contract_class();

                let (checker_address, _) = checker.deploy(@array![]).unwrap();
                let (proxy_address, _) = proxy.deploy(@array![checker_address.into()]).unwrap();
                let (dummy_address, _) = dummy.deploy(@array![]).unwrap();

                let proxy_dispatcher = ITraceInfoProxyDispatcher { contract_address: proxy_address };

                proxy_dispatcher.regular_call(checker_address);
                proxy_dispatcher.with_libcall(checker.class_hash);
                proxy_dispatcher.call_two(checker_address, dummy_address);

                let chcecker_dispatcher = ITraceInfoCheckerDispatcher { contract_address: checker_address };
                chcecker_dispatcher.from_proxy(4);

                assert_trace(
                    get_call_trace(), proxy_address, checker_address, dummy_address, checker.class_hash
                );
            }

            fn assert_trace(
                trace: CallTrace,
                proxy_address: ContractAddress,
                checker_address: ContractAddress,
                dummy_address: ContractAddress,
                checker_class_hash: ClassHash
            ) {
                let expected = CallTrace {
                    entry_point: CallEntryPoint {
                        entry_point_type: EntryPointType::External,
                        entry_point_selector: test_selector(),
                        calldata: array![],
                        contract_address: test_address(),
                        caller_address: 0.try_into().unwrap(),
                        call_type: CallType::Call,
                    },
                    nested_calls: array![
                        CallTrace {
                            entry_point: CallEntryPoint {
                                entry_point_type: EntryPointType::Constructor,
                                entry_point_selector: selector!("constructor"),
                                calldata: array![checker_address.into()],
                                contract_address: proxy_address,
                                caller_address: test_address(),
                                call_type: CallType::Call,
                            },
                            nested_calls: array![
                                CallTrace {
                                    entry_point: CallEntryPoint {
                                        entry_point_type: EntryPointType::External,
                                        entry_point_selector: selector!("from_proxy"),
                                        calldata: array![1],
                                        contract_address: checker_address,
                                        caller_address: proxy_address,
                                        call_type: CallType::Call,
                                    },
                                    nested_calls: array![],
                                    result: CallResult::Success(array![101])
                                },
                            ],
                            result: CallResult::Success(array![])
                        },
                        CallTrace {
                            entry_point: CallEntryPoint {
                                entry_point_type: EntryPointType::External,
                                entry_point_selector: selector!("regular_call"),
                                calldata: array![checker_address.into()],
                                contract_address: proxy_address,
                                caller_address: test_address(),
                                call_type: CallType::Call,
                            },
                            nested_calls: array![
                                CallTrace {
                                    entry_point: CallEntryPoint {
                                        entry_point_type: EntryPointType::External,
                                        entry_point_selector: selector!("from_proxy"),
                                        calldata: array![2],
                                        contract_address: checker_address,
                                        caller_address: proxy_address,
                                        call_type: CallType::Call,
                                    },
                                    nested_calls: array![],
                                    result: CallResult::Success(array![102])
                                }
                            ],
                            result: CallResult::Success(array![102])
                        },
                        CallTrace {
                            entry_point: CallEntryPoint {
                                entry_point_type: EntryPointType::External,
                                entry_point_selector: selector!("with_libcall"),
                                calldata: array![checker_class_hash.into()],
                                contract_address: proxy_address,
                                caller_address: test_address(),
                                call_type: CallType::Call,
                            },
                            nested_calls: array![
                                CallTrace {
                                    entry_point: CallEntryPoint {
                                        entry_point_type: EntryPointType::External,
                                        entry_point_selector: selector!("from_proxy"),
                                        calldata: array![3],
                                        contract_address: proxy_address,
                                        caller_address: test_address(),
                                        call_type: CallType::Delegate,
                                    },
                                    nested_calls: array![],
                                    result: CallResult::Success(array![103])
                                }
                            ],
                            result: CallResult::Success(array![103])
                        },
                        CallTrace {
                            entry_point: CallEntryPoint {
                                entry_point_type: EntryPointType::External,
                                entry_point_selector: selector!("call_two"),
                                calldata: array![checker_address.into(), dummy_address.into()],
                                contract_address: proxy_address,
                                caller_address: test_address(),
                                call_type: CallType::Call,
                            },
                            nested_calls: array![
                                CallTrace {
                                    entry_point: CallEntryPoint {
                                        entry_point_type: EntryPointType::External,
                                        entry_point_selector: selector!("from_proxy"),
                                        calldata: array![42],
                                        contract_address: checker_address,
                                        caller_address: proxy_address,
                                        call_type: CallType::Call,
                                    },
                                    nested_calls: array![],
                                    result: CallResult::Success(array![142])
                                },
                                CallTrace {
                                    entry_point: CallEntryPoint {
                                        entry_point_type: EntryPointType::External,
                                        entry_point_selector: selector!("from_proxy_dummy"),
                                        calldata: array![],
                                        contract_address: dummy_address,
                                        caller_address: proxy_address,
                                        call_type: CallType::Call,
                                    },
                                    nested_calls: array![],
                                    result: CallResult::Success(array![])
                                }
                            ],
                            result: CallResult::Success(array![])
                        },
                        CallTrace {
                            entry_point: CallEntryPoint {
                                entry_point_type: EntryPointType::External,
                                entry_point_selector: selector!("from_proxy"),
                                calldata: array![4],
                                contract_address: checker_address,
                                caller_address: test_address(),
                                call_type: CallType::Call,
                            },
                            nested_calls: array![],
                            result: CallResult::Success(array![104])
                        }
                    ],
                    result: CallResult::Success(array![])
                };

                assert(expected == trace, '');
            }
        "#
        ),
        Contract::from_code_path(
            "TraceInfoProxy".to_string(),
            Path::new("tests/data/contracts/trace_info_proxy.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "TraceInfoChecker".to_string(),
            Path::new("tests/data/contracts/trace_info_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "TraceDummy".to_string(),
            Path::new("tests/data/contracts/trace_dummy.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
#[expect(clippy::too_many_lines)]
fn trace_failed_call() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{declare, ContractClassTrait, DeclareResultTrait, test_address, test_selector};
            use snforge_std::trace::{CallEntryPoint, CallType, EntryPointType, get_call_trace, CallTrace, CallResult, CallFailure};

            use starknet::{ContractAddress, ClassHash};

            #[starknet::interface]
            trait ITraceInfoProxy<T> {
                fn with_libcall(self: @T, class_hash: ClassHash) -> felt252;
                fn regular_call(self: @T, contract_address: ContractAddress) -> felt252;
                fn with_panic(self: @T, contract_address: ContractAddress);
            }

            #[starknet::interface]
            trait ITraceInfoChecker<T> {
                fn from_proxy(self: @T, data: felt252) -> felt252;
                fn panic(self: @T);
            }

            #[test]
            #[feature("safe_dispatcher")]
            fn test_failed_call_trace_info() {
                let proxy = declare("TraceInfoProxy").unwrap().contract_class();
                let checker = declare("TraceInfoChecker").unwrap().contract_class();

                let (checker_address, _) = checker.deploy(@array![]).unwrap();
                let (proxy_address, _) = proxy.deploy(@array![checker_address.into()]).unwrap();

                let proxy_dispatcher = ITraceInfoProxySafeDispatcher { contract_address: proxy_address };
                match proxy_dispatcher.with_panic(checker_address) {
                    Result::Ok(_) => panic_with_felt252('shouldve panicked'),
                    Result::Err(panic_data) => { assert(*panic_data.at(0) == 'panic', *panic_data.at(0)); }
                }

                let chcecker_dispatcher = ITraceInfoCheckerSafeDispatcher { contract_address: checker_address };
                match chcecker_dispatcher.panic() {
                    Result::Ok(_) => panic_with_felt252('shouldve panicked'),
                    Result::Err(panic_data) => { assert(*panic_data.at(0) == 'panic', *panic_data.at(0)); }
                }

                assert_trace(get_call_trace(), proxy_address, checker_address);
            }

            fn assert_trace(
                trace: CallTrace, proxy_address: ContractAddress, checker_address: ContractAddress
            ) {
                let expected = CallTrace {
                    entry_point: CallEntryPoint {
                        entry_point_type: EntryPointType::External,
                        entry_point_selector: test_selector(),
                        calldata: array![],
                        contract_address: test_address(),
                        caller_address: 0.try_into().unwrap(),
                        call_type: CallType::Call,
                    },
                    nested_calls: array![
                        CallTrace {
                            entry_point: CallEntryPoint {
                                entry_point_type: EntryPointType::Constructor,
                                entry_point_selector: selector!("constructor"),
                                calldata: array![checker_address.into()],
                                contract_address: proxy_address,
                                caller_address: test_address(),
                                call_type: CallType::Call,
                            },
                            nested_calls: array![
                                CallTrace {
                                    entry_point: CallEntryPoint {
                                        entry_point_type: EntryPointType::External,
                                        entry_point_selector: selector!("from_proxy"),
                                        calldata: array![1],
                                        contract_address: checker_address,
                                        caller_address: proxy_address,
                                        call_type: CallType::Call,
                                    },
                                    nested_calls: array![],
                                    result: CallResult::Success(array![101])
                                },
                            ],
                            result: CallResult::Success(array![])
                        },
                        CallTrace {
                            entry_point: CallEntryPoint {
                                entry_point_type: EntryPointType::External,
                                entry_point_selector: selector!("with_panic"),
                                calldata: array![checker_address.into()],
                                contract_address: proxy_address,
                                caller_address: test_address(),
                                call_type: CallType::Call,
                            },
                            nested_calls: array![
                                CallTrace {
                                    entry_point: CallEntryPoint {
                                        entry_point_type: EntryPointType::External,
                                        entry_point_selector: selector!("panic"),
                                        calldata: array![],
                                        contract_address: checker_address,
                                        caller_address: proxy_address,
                                        call_type: CallType::Call,
                                    },
                                    nested_calls: array![],
                                    result: CallResult::Failure(CallFailure::Panic(array![482670963043]))
                                }
                            ],
                            result: CallResult::Failure(CallFailure::Panic(array![482670963043]))
                        },
                        CallTrace {
                            entry_point: CallEntryPoint {
                                entry_point_type: EntryPointType::External,
                                entry_point_selector: selector!("panic"),
                                calldata: array![],
                                contract_address: checker_address,
                                caller_address: test_address(),
                                call_type: CallType::Call,
                            },
                            nested_calls: array![],
                            result: CallResult::Failure(CallFailure::Panic(array![482670963043]))
                        }
                    ],
                    result: CallResult::Success(array![])
                };

                assert(expected == trace, '');
            }
        "#
        ),
        Contract::from_code_path(
            "TraceInfoProxy".to_string(),
            Path::new("tests/data/contracts/trace_info_proxy.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "TraceInfoChecker".to_string(),
            Path::new("tests/data/contracts/trace_info_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
#[expect(clippy::too_many_lines)]
fn trace_library_call_from_test() {
    let test = test_case!(
        indoc!(
            r#"
            use core::clone::Clone;
            use snforge_std::{declare, ContractClassTrait, DeclareResultTrait, test_address, test_selector};
            use snforge_std::trace::{CallEntryPoint, CallType, EntryPointType, get_call_trace, CallTrace, CallResult};

            use starknet::{ContractAddress, ClassHash};

            #[starknet::interface]
            trait ITraceInfoProxy<T> {
                fn with_libcall(self: @T, class_hash: ClassHash) -> felt252;
                fn regular_call(self: @T, contract_address: ContractAddress) -> felt252;
                fn with_panic(self: @T, contract_address: ContractAddress);
                fn call_two(self: @T, checker_address: ContractAddress, dummy_address: ContractAddress);
            }

            #[starknet::interface]
            trait ITraceInfoChecker<T> {
                fn from_proxy(self: @T, data: felt252) -> felt252;
                fn panic(self: @T);
            }

            #[starknet::interface]
            trait ITraceDummy<T> {
                fn from_proxy(ref self: T);
            }

            #[test]
            fn test_library_call_trace_info() {
                let proxy_hash = declare("TraceInfoProxy").unwrap().contract_class().class_hash.clone();
                let checker = declare("TraceInfoChecker").unwrap().contract_class().clone();
                let dummy = declare("TraceDummy").unwrap().contract_class();

                let (checker_address, _) = checker.deploy(@array![]).unwrap();
                let (dummy_address, _) = dummy.deploy(@array![]).unwrap();

                let proxy_lib_dispatcher = ITraceInfoProxyLibraryDispatcher { class_hash: proxy_hash };

                proxy_lib_dispatcher.regular_call(checker_address);
                proxy_lib_dispatcher.with_libcall(checker.class_hash);
                proxy_lib_dispatcher.call_two(checker_address, dummy_address);

                let chcecker_lib_dispatcher = ITraceInfoCheckerLibraryDispatcher {
                    class_hash: checker.class_hash
                };

                chcecker_lib_dispatcher.from_proxy(4);

                assert_trace(get_call_trace(), checker_address, dummy_address, checker.class_hash);
            }

            fn assert_trace(
                trace: CallTrace,
                checker_address: ContractAddress,
                dummy_address: ContractAddress,
                checker_class_hash: ClassHash
            ) {
                let expected = CallTrace {
                    entry_point: CallEntryPoint {
                        entry_point_type: EntryPointType::External,
                        entry_point_selector: test_selector(),
                        calldata: array![],
                        contract_address: test_address(),
                        caller_address: 0.try_into().unwrap(),
                        call_type: CallType::Call,
                    },
                    nested_calls: array![
                        CallTrace {
                            entry_point: CallEntryPoint {
                                entry_point_type: EntryPointType::External,
                                entry_point_selector: selector!("regular_call"),
                                calldata: array![checker_address.into()],
                                contract_address: test_address(),
                                caller_address: 0.try_into().unwrap(),
                                call_type: CallType::Delegate,
                            },
                            nested_calls: array![
                                CallTrace {
                                    entry_point: CallEntryPoint {
                                        entry_point_type: EntryPointType::External,
                                        entry_point_selector: selector!("from_proxy"),
                                        calldata: array![2],
                                        contract_address: checker_address,
                                        caller_address: test_address(),
                                        call_type: CallType::Call,
                                    },
                                    nested_calls: array![],
                                    result: CallResult::Success(array![102])
                                }
                            ],
                            result: CallResult::Success(array![102])
                        },
                        CallTrace {
                            entry_point: CallEntryPoint {
                                entry_point_type: EntryPointType::External,
                                entry_point_selector: selector!("with_libcall"),
                                calldata: array![checker_class_hash.into()],
                                contract_address: test_address(),
                                caller_address: 0.try_into().unwrap(),
                                call_type: CallType::Delegate,
                            },
                            nested_calls: array![
                                CallTrace {
                                    entry_point: CallEntryPoint {
                                        entry_point_type: EntryPointType::External,
                                        entry_point_selector: selector!("from_proxy"),
                                        calldata: array![3],
                                        contract_address: test_address(),
                                        caller_address: 0.try_into().unwrap(),
                                        call_type: CallType::Delegate,
                                    },
                                    nested_calls: array![],
                                    result: CallResult::Success(array![103])
                                }
                            ],
                            result: CallResult::Success(array![103])
                        },
                        CallTrace {
                            entry_point: CallEntryPoint {
                                entry_point_type: EntryPointType::External,
                                entry_point_selector: selector!("call_two"),
                                calldata: array![checker_address.into(), dummy_address.into()],
                                contract_address: test_address(),
                                caller_address: 0.try_into().unwrap(),
                                call_type: CallType::Delegate,
                            },
                            nested_calls: array![
                                CallTrace {
                                    entry_point: CallEntryPoint {
                                        entry_point_type: EntryPointType::External,
                                        entry_point_selector: selector!("from_proxy"),
                                        calldata: array![42],
                                        contract_address: checker_address,
                                        caller_address: test_address(),
                                        call_type: CallType::Call,
                                    },
                                    nested_calls: array![],
                                    result: CallResult::Success(array![142])
                                },
                                CallTrace {
                                    entry_point: CallEntryPoint {
                                        entry_point_type: EntryPointType::External,
                                        entry_point_selector: selector!("from_proxy_dummy"),
                                        calldata: array![],
                                        contract_address: dummy_address,
                                        caller_address: test_address(),
                                        call_type: CallType::Call,
                                    },
                                    nested_calls: array![],
                                    result: CallResult::Success(array![])
                                }
                            ],
                            result: CallResult::Success(array![])
                        },
                        CallTrace {
                            entry_point: CallEntryPoint {
                                entry_point_type: EntryPointType::External,
                                entry_point_selector: selector!("from_proxy"),
                                calldata: array![4],
                                contract_address: test_address(),
                                caller_address: 0.try_into().unwrap(),
                                call_type: CallType::Delegate,
                            },
                            nested_calls: array![],
                            result: CallResult::Success(array![104])
                        }
                    ],
                    result: CallResult::Success(array![])
                };

                assert(expected == trace, '');
            }
        "#
        ),
        Contract::from_code_path(
            "TraceInfoProxy".to_string(),
            Path::new("tests/data/contracts/trace_info_proxy.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "TraceInfoChecker".to_string(),
            Path::new("tests/data/contracts/trace_info_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "TraceDummy".to_string(),
            Path::new("tests/data/contracts/trace_dummy.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
#[expect(clippy::too_many_lines)]
fn trace_failed_library_call_from_test() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{declare, ContractClassTrait, DeclareResultTrait, test_address, test_selector};
            use snforge_std::trace::{CallEntryPoint, CallType, EntryPointType, get_call_trace, CallTrace, CallResult, CallFailure};

            use starknet::{ContractAddress, ClassHash};

            #[starknet::interface]
            trait ITraceInfoProxy<T> {
                fn with_libcall(self: @T, class_hash: ClassHash) -> felt252;
                fn regular_call(self: @T, contract_address: ContractAddress) -> felt252;
                fn with_panic(self: @T, contract_address: ContractAddress);
            }

            #[starknet::interface]
            trait ITraceInfoChecker<T> {
                fn from_proxy(self: @T, data: felt252) -> felt252;
                fn panic(self: @T);
            }

            #[test]
            #[feature("safe_dispatcher")]
            fn test_failed_call_trace_info() {
                let proxy = declare("TraceInfoProxy").unwrap().contract_class();
                let checker = declare("TraceInfoChecker").unwrap().contract_class();

                let (checker_address, _) = checker.deploy(@array![]).unwrap();
                let (proxy_address, _) = proxy.deploy(@array![checker_address.into()]).unwrap();

                let proxy_dispatcher = ITraceInfoProxySafeDispatcher { contract_address: proxy_address };
                match proxy_dispatcher.with_panic(checker_address) {
                    Result::Ok(_) => panic_with_felt252('shouldve panicked'),
                    Result::Err(panic_data) => { assert(*panic_data.at(0) == 'panic', *panic_data.at(0)); }
                }

                let chcecker_dispatcher = ITraceInfoCheckerSafeDispatcher { contract_address: checker_address };
                match chcecker_dispatcher.panic() {
                    Result::Ok(_) => panic_with_felt252('shouldve panicked'),
                    Result::Err(panic_data) => { assert(*panic_data.at(0) == 'panic', *panic_data.at(0)); }
                }

                assert_trace(get_call_trace(), proxy_address, checker_address);
            }

            fn assert_trace(
                trace: CallTrace, proxy_address: ContractAddress, checker_address: ContractAddress
            ) {
                let expected = CallTrace {
                    entry_point: CallEntryPoint {
                        entry_point_type: EntryPointType::External,
                        entry_point_selector: test_selector(),
                        calldata: array![],
                        contract_address: test_address(),
                        caller_address: 0.try_into().unwrap(),
                        call_type: CallType::Call,
                    },
                    nested_calls: array![
                        CallTrace {
                            entry_point: CallEntryPoint {
                                entry_point_type: EntryPointType::Constructor,
                                entry_point_selector: selector!("constructor"),
                                calldata: array![checker_address.into()],
                                contract_address: proxy_address,
                                caller_address: test_address(),
                                call_type: CallType::Call,
                            },
                            nested_calls: array![
                                CallTrace {
                                    entry_point: CallEntryPoint {
                                        entry_point_type: EntryPointType::External,
                                        entry_point_selector: selector!("from_proxy"),
                                        calldata: array![1],
                                        contract_address: checker_address,
                                        caller_address: proxy_address,
                                        call_type: CallType::Call,
                                    },
                                    nested_calls: array![],
                                    result: CallResult::Success(array![101])
                                },
                            ],
                            result: CallResult::Success(array![])
                        },
                        CallTrace {
                            entry_point: CallEntryPoint {
                                entry_point_type: EntryPointType::External,
                                entry_point_selector: selector!("with_panic"),
                                calldata: array![checker_address.into()],
                                contract_address: proxy_address,
                                caller_address: test_address(),
                                call_type: CallType::Call,
                            },
                            nested_calls: array![
                                CallTrace {
                                    entry_point: CallEntryPoint {
                                        entry_point_type: EntryPointType::External,
                                        entry_point_selector: selector!("panic"),
                                        calldata: array![],
                                        contract_address: checker_address,
                                        caller_address: proxy_address,
                                        call_type: CallType::Call,
                                    },
                                    nested_calls: array![],
                                    result: CallResult::Failure(CallFailure::Panic(array![482670963043]))
                                }
                            ],
                            result: CallResult::Failure(CallFailure::Panic(array![482670963043]))
                        },
                        CallTrace {
                            entry_point: CallEntryPoint {
                                entry_point_type: EntryPointType::External,
                                entry_point_selector: selector!("panic"),
                                calldata: array![],
                                contract_address: checker_address,
                                caller_address: test_address(),
                                call_type: CallType::Call,
                            },
                            nested_calls: array![],
                            result: CallResult::Failure(CallFailure::Panic(array![482670963043]))
                        }
                    ],
                    result: CallResult::Success(array![])
                };

                assert(expected == trace, '');
            }
        "#
        ),
        Contract::from_code_path(
            "TraceInfoProxy".to_string(),
            Path::new("tests/data/contracts/trace_info_proxy.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "TraceInfoChecker".to_string(),
            Path::new("tests/data/contracts/trace_info_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
#[expect(clippy::too_many_lines)]
fn trace_l1_handler() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{declare, ContractClassTrait, DeclareResultTrait, test_address, test_selector, L1HandlerTrait,};
            use snforge_std::trace::{CallEntryPoint, CallType, EntryPointType, get_call_trace, CallTrace, CallResult};

            use starknet::ContractAddress;

            #[test]
            fn test_l1_handler_call_trace_info() {
                let proxy = declare("TraceInfoProxy").unwrap().contract_class();
                let checker = declare("TraceInfoChecker").unwrap().contract_class();

                let (checker_address, _) = checker.deploy(@array![]).unwrap();
                let (proxy_address, _) = proxy.deploy(@array![checker_address.into()]).unwrap();

                let mut l1_handler = L1HandlerTrait::new(checker_address, selector!("handle_l1_message"));

                l1_handler.execute(123, array![proxy_address.into()].span()).unwrap();
                assert_trace(get_call_trace(), proxy_address, checker_address);
            }

            fn assert_trace(
                trace: CallTrace, proxy_address: ContractAddress, checker_address: ContractAddress
            ) {
                let expected_trace = CallTrace {
                    entry_point: CallEntryPoint {
                        entry_point_type: EntryPointType::External,
                        entry_point_selector: test_selector(),
                        calldata: array![],
                        contract_address: test_address(),
                        caller_address: 0.try_into().unwrap(),
                        call_type: CallType::Call,
                    },
                    nested_calls: array![
                        CallTrace {
                            entry_point: CallEntryPoint {
                                entry_point_type: EntryPointType::Constructor,
                                entry_point_selector: selector!("constructor"),
                                calldata: array![checker_address.into()],
                                contract_address: proxy_address,
                                caller_address: test_address(),
                                call_type: CallType::Call,
                            },
                            nested_calls: array![
                                CallTrace {
                                    entry_point: CallEntryPoint {
                                        entry_point_type: EntryPointType::External,
                                        entry_point_selector: selector!("from_proxy"),
                                        calldata: array![1],
                                        contract_address: checker_address,
                                        caller_address: proxy_address,
                                        call_type: CallType::Call,
                                    },
                                    nested_calls: array![],
                                    result: CallResult::Success(array![101])
                                }
                            ],
                            result: CallResult::Success(array![])
                        },
                        CallTrace {
                            entry_point: CallEntryPoint {
                                entry_point_type: EntryPointType::L1Handler,
                                entry_point_selector: selector!("handle_l1_message"),
                                calldata: array![123, proxy_address.into()],
                                contract_address: checker_address,
                                caller_address: 0.try_into().unwrap(),
                                call_type: CallType::Call,
                            },
                            nested_calls: array![
                                CallTrace {
                                    entry_point: CallEntryPoint {
                                        entry_point_type: EntryPointType::External,
                                        entry_point_selector: selector!("regular_call"),
                                        calldata: array![checker_address.into()],
                                        contract_address: proxy_address,
                                        caller_address: checker_address,
                                        call_type: CallType::Call,
                                    },
                                    nested_calls: array![
                                        CallTrace {
                                            entry_point: CallEntryPoint {
                                                entry_point_type: EntryPointType::External,
                                                entry_point_selector: selector!("from_proxy"),
                                                calldata: array![2],
                                                contract_address: checker_address,
                                                caller_address: proxy_address,
                                                call_type: CallType::Call,
                                            },
                                            nested_calls: array![],
                                            result: CallResult::Success(array![102])
                                        }
                                    ],
                                    result: CallResult::Success(array![102])
                                }
                            ],
                            result: CallResult::Success(array![102])
                        }
                    ],
                    result: CallResult::Success(array![])
                };

                assert(trace == expected_trace, '');
            }
        "#
        ),
        Contract::from_code_path(
            "TraceInfoProxy".to_string(),
            Path::new("tests/data/contracts/trace_info_proxy.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "TraceInfoChecker".to_string(),
            Path::new("tests/data/contracts/trace_info_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}
