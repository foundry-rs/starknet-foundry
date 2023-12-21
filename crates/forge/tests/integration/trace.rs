use indoc::indoc;
use std::path::Path;
use test_utils::runner::Contract;
use test_utils::running_tests::run_test_case;
use test_utils::{assert_passed, test_case};

#[test]
fn trace_deploy() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{declare, ContractClassTrait, test_address};
            use snforge_std::trace::{CallEntryPoint, CallType, EntryPointType, get_last_call_trace};
            
            use starknet::{SyscallResultTrait, deploy_syscall, ContractAddress};
            
            #[starknet::interface]
            trait ITraceInfoProxy<T> {
                fn with_libcall(ref self: T, class_hash: starknet::ClassHash) -> felt252;
                fn regular_call(self: @T, contract_address: starknet::ContractAddress) -> felt252;
            }
            
            #[starknet::interface]
            trait ITraceInfoChecker<T> {
                fn from_proxy(self: @T, a_data: felt252) -> felt252;
            }
            
            const CONSTRUCTOR_SELECTOR: felt252 =
                1159040026212278395030414237414753050475174923702621880048416706425641521556;
            
            #[test]
            fn test_deploy_trace_info() {
                assert_eq!(get_last_call_trace().len(), 0);
            
                let proxy = declare('TraceInfoProxy');
                assert_eq!(get_last_call_trace().len(), 0);
            
                let checker = declare('TraceInfoChecker');
                assert_eq!(get_last_call_trace().len(), 0);
            
                let checker_address = checker.deploy(@array![]).unwrap();
                assert_eq!(get_last_call_trace().len(), 0); // no constructor
            
                let proxy_address = proxy.deploy(@array![checker_address.into()]).unwrap();
                assert_trace_after_proxy_deploy(get_last_call_trace(), proxy_address, checker_address);
            
                let (proxy_address_2, _) = deploy_syscall(
                    proxy.class_hash, 0, array![checker_address.into()].span(), false
                )
                    .unwrap_syscall();
                assert_trace_after_proxy_deploy(get_last_call_trace(), proxy_address_2, checker_address);
            
                let proxy_address_3 = proxy
                    .deploy_at(@array![checker_address.into()], 123.try_into().unwrap())
                    .unwrap();
                assert_trace_after_proxy_deploy(get_last_call_trace(), proxy_address_3, checker_address);
            }
            
            fn assert_trace_after_proxy_deploy(
                trace: Array<CallEntryPoint>, proxy_address: ContractAddress, checker_address: ContractAddress
            ) {
                let expected_trace = array![
                    CallEntryPoint {
                        entry_point_type: EntryPointType::Constructor,
                        entry_point_selector: CONSTRUCTOR_SELECTOR,
                        calldata: array![checker_address.into()],
                        storage_address: proxy_address,
                        caller_address: test_address(),
                        call_type: CallType::Call,
                    },
                    CallEntryPoint {
                        entry_point_type: EntryPointType::External,
                        entry_point_selector: selector!("from_proxy"),
                        calldata: array![1],
                        storage_address: checker_address,
                        caller_address: proxy_address,
                        call_type: CallType::Call,
                    },
                ];
            
                assert(trace == expected_trace, 'proxy deploy');
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

    assert_passed!(result);
}

#[test]
#[allow(clippy::too_many_lines)]
fn trace_call() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{declare, ContractClassTrait, test_address};
            use snforge_std::trace::{CallEntryPoint, CallType, EntryPointType, get_last_call_trace};
            
            use starknet::{ContractAddress, ClassHash};
            
            #[starknet::interface]
            trait ITraceInfoProxy<T> {
                fn with_libcall(ref self: T, class_hash: ClassHash) -> felt252;
                fn regular_call(self: @T, contract_address: ContractAddress) -> felt252;
                fn with_panic(self: @T, contract_address: ContractAddress);
            }
            
            #[starknet::interface]
            trait ITraceInfoChecker<T> {
                fn from_proxy(self: @T, data: felt252) -> felt252;
                fn panic(self: @T);
            }
            
            #[test]
            fn test_call_trace_info() {
                let proxy = declare('TraceInfoProxy');
                let checker = declare('TraceInfoChecker');
            
                let checker_address = checker.deploy(@array![]).unwrap();
                let proxy_address = proxy.deploy(@array![checker_address.into()]).unwrap();
            
                let proxy_dispatcher = ITraceInfoProxyDispatcher { contract_address: proxy_address };
            
                proxy_dispatcher.regular_call(checker_address);
                assert_trace_after_proxy_regular_call(get_last_call_trace(), proxy_address, checker_address);
            
                proxy_dispatcher.with_libcall(checker.class_hash);
                assert_trace_after_proxy_call_with_libcall(
                    get_last_call_trace(), proxy_address, checker.class_hash
                );
            
                let chcecker_dispatcher = ITraceInfoCheckerDispatcher { contract_address: checker_address };
            
                chcecker_dispatcher.from_proxy(4);
                assert_trace_after_checker_call_from_test(get_last_call_trace(), checker_address);
            }
            
            fn assert_trace_after_proxy_regular_call(
                trace: Array<CallEntryPoint>, proxy_address: ContractAddress, checker_address: ContractAddress
            ) {
                let expected_trace = array![
                    CallEntryPoint {
                        entry_point_type: EntryPointType::External,
                        entry_point_selector: selector!("regular_call"),
                        calldata: array![checker_address.into()],
                        storage_address: proxy_address,
                        caller_address: test_address(),
                        call_type: CallType::Call,
                    },
                    CallEntryPoint {
                        entry_point_type: EntryPointType::External,
                        entry_point_selector: selector!("from_proxy"),
                        calldata: array![2],
                        storage_address: checker_address,
                        caller_address: proxy_address,
                        call_type: CallType::Call,
                    },
                ];
            
                assert(trace == expected_trace, 'proxy regular_call');
            }
            
            fn assert_trace_after_proxy_call_with_libcall(
                trace: Array<CallEntryPoint>, proxy_address: ContractAddress, checker_class_hash: ClassHash
            ) {
                let expected_trace = array![
                    CallEntryPoint {
                        entry_point_type: EntryPointType::External,
                        entry_point_selector: selector!("with_libcall"),
                        calldata: array![checker_class_hash.into()],
                        storage_address: proxy_address,
                        caller_address: test_address(),
                        call_type: CallType::Call,
                    },
                    CallEntryPoint {
                        entry_point_type: EntryPointType::External,
                        entry_point_selector: selector!("from_proxy"),
                        calldata: array![3],
                        storage_address: proxy_address,
                        caller_address: test_address(),
                        call_type: CallType::Delegate,
                    },
                ];
            
                assert(trace == expected_trace, 'proxy with_libcall');
            }
            
            fn assert_trace_after_checker_call_from_test(
                trace: Array<CallEntryPoint>, checker_address: ContractAddress
            ) {
                let expected_trace = array![
                    CallEntryPoint {
                        entry_point_type: EntryPointType::External,
                        entry_point_selector: selector!("from_proxy"),
                        calldata: array![4],
                        storage_address: checker_address,
                        caller_address: test_address(),
                        call_type: CallType::Call,
                    },
                ];
            
                assert(trace == expected_trace, 'checker from_proxy');
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

    assert_passed!(result);
}

#[test]
fn trace_failed_call() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{declare, ContractClassTrait, test_address};
            use snforge_std::trace::{CallEntryPoint, CallType, EntryPointType, get_last_call_trace};
            
            use starknet::{ContractAddress, ClassHash};
            
            #[starknet::interface]
            trait ITraceInfoProxy<T> {
                fn with_libcall(ref self: T, class_hash: ClassHash) -> felt252;
                fn regular_call(self: @T, contract_address: ContractAddress) -> felt252;
                fn with_panic(self: @T, contract_address: ContractAddress);
            }
            
            #[starknet::interface]
            trait ITraceInfoChecker<T> {
                fn from_proxy(self: @T, data: felt252) -> felt252;
                fn panic(self: @T);
            }
            
            #[test]
            fn test_failed_call_trace_info() {
                let proxy = declare('TraceInfoProxy');
                let checker = declare('TraceInfoChecker');
            
                let checker_address = checker.deploy(@array![]).unwrap();
                let proxy_address = proxy.deploy(@array![checker_address.into()]).unwrap();
            
                let proxy_dispatcher = ITraceInfoProxySafeDispatcher { contract_address: proxy_address };
            
                match proxy_dispatcher.with_panic(checker_address) {
                    Result::Ok(_) => panic_with_felt252('shouldve panicked'),
                    Result::Err(panic_data) => {
                        assert(*panic_data.at(0) == 'panic', *panic_data.at(0));
                        assert_trace_after_proxy_with_panic_call(
                            get_last_call_trace(), proxy_address, checker_address
                        );
                    }
                }
            
                let chcecker_dispatcher = ITraceInfoCheckerSafeDispatcher { contract_address: checker_address };
            
                match chcecker_dispatcher.panic() {
                    Result::Ok(_) => panic_with_felt252('shouldve panicked'),
                    Result::Err(panic_data) => {
                        assert(*panic_data.at(0) == 'panic', *panic_data.at(0));
                        assert_trace_after_checker_panic_call_from_test(get_last_call_trace(), checker_address);
                    }
                }
            }
            
            fn assert_trace_after_checker_panic_call_from_test(
                trace: Array<CallEntryPoint>, checker_address: ContractAddress
            ) {
                let expected_trace = array![
                    CallEntryPoint {
                        entry_point_type: EntryPointType::External,
                        entry_point_selector: selector!("panic"),
                        calldata: array![],
                        storage_address: checker_address,
                        caller_address: test_address(),
                        call_type: CallType::Call,
                    },
                ];
            
                assert(trace == expected_trace, 'checker panic');
            }
            
            fn assert_trace_after_proxy_with_panic_call(
                trace: Array<CallEntryPoint>, proxy_address: ContractAddress, checker_address: ContractAddress
            ) {
                let expected_trace = array![
                    CallEntryPoint {
                        entry_point_type: EntryPointType::External,
                        entry_point_selector: selector!("with_panic"),
                        calldata: array![checker_address.into()],
                        storage_address: proxy_address,
                        caller_address: test_address(),
                        call_type: CallType::Call,
                    },
                    CallEntryPoint {
                        entry_point_type: EntryPointType::External,
                        entry_point_selector: selector!("panic"),
                        calldata: array![],
                        storage_address: checker_address,
                        caller_address: proxy_address,
                        call_type: CallType::Call,
                    },
                ];
            
                assert(trace == expected_trace, 'proxy with_panic');
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

    assert_passed!(result);
}

#[test]
#[allow(clippy::too_many_lines)]
fn trace_library_call_from_test() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{declare, ContractClassTrait, test_address};
            use snforge_std::trace::{CallEntryPoint, CallType, EntryPointType, get_last_call_trace};
            
            use starknet::{ContractAddress, ClassHash};
            
            #[starknet::interface]
            trait ITraceInfoProxy<T> {
                fn with_libcall(ref self: T, class_hash: ClassHash) -> felt252;
                fn regular_call(self: @T, contract_address: ContractAddress) -> felt252;
                fn with_panic(self: @T, contract_address: ContractAddress);
            }
            
            #[starknet::interface]
            trait ITraceInfoChecker<T> {
                fn from_proxy(self: @T, data: felt252) -> felt252;
                fn panic(self: @T);
            }
            
            #[test]
            fn test_library_call_trace_info() {
                let proxy_hash = declare('TraceInfoProxy').class_hash;
                let checker = declare('TraceInfoChecker');
                let checker_address = checker.deploy(@array![]).unwrap();
            
                let proxy_lib_dispatcher = ITraceInfoProxyLibraryDispatcher { class_hash: proxy_hash };
            
                proxy_lib_dispatcher.regular_call(checker_address);
                assert_trace_after_proxy_regular_call(get_last_call_trace(), checker_address);
            
                proxy_lib_dispatcher.with_libcall(checker.class_hash);
                assert_trace_after_proxy_call_with_libcall(get_last_call_trace(), checker.class_hash);
            
                let chcecker_lib_dispatcher = ITraceInfoCheckerLibraryDispatcher {
                    class_hash: checker.class_hash
                };
            
                chcecker_lib_dispatcher.from_proxy(4);
                assert_trace_after_checker_call_from_test(get_last_call_trace());
            }
            
            fn assert_trace_after_proxy_regular_call(
                trace: Array<CallEntryPoint>, checker_address: ContractAddress
            ) {
                let expected_trace = array![
                    CallEntryPoint {
                        entry_point_type: EntryPointType::External,
                        entry_point_selector: selector!("regular_call"),
                        calldata: array![checker_address.into()],
                        storage_address: test_address(),
                        caller_address: 0.try_into().unwrap(),
                        call_type: CallType::Delegate,
                    },
                    CallEntryPoint {
                        entry_point_type: EntryPointType::External,
                        entry_point_selector: selector!("from_proxy"),
                        calldata: array![2],
                        storage_address: checker_address,
                        caller_address: test_address(),
                        call_type: CallType::Call,
                    },
                ];
            
                assert(trace == expected_trace, 'proxy libcall regular_call');
            }
            
            fn assert_trace_after_proxy_call_with_libcall(
                trace: Array<CallEntryPoint>, checker_class_hash: ClassHash
            ) {
                let expected_trace = array![
                    CallEntryPoint {
                        entry_point_type: EntryPointType::External,
                        entry_point_selector: selector!("with_libcall"),
                        calldata: array![checker_class_hash.into()],
                        storage_address: test_address(),
                        caller_address: 0.try_into().unwrap(),
                        call_type: CallType::Delegate,
                    },
                    CallEntryPoint {
                        entry_point_type: EntryPointType::External,
                        entry_point_selector: selector!("from_proxy"),
                        calldata: array![3],
                        storage_address: test_address(),
                        caller_address: 0.try_into().unwrap(),
                        call_type: CallType::Delegate,
                    },
                ];
            
                assert(trace == expected_trace, 'proxy libcall with_libcall');
            }
            
            fn assert_trace_after_checker_call_from_test(trace: Array<CallEntryPoint>) {
                let expected_trace = array![
                    CallEntryPoint {
                        entry_point_type: EntryPointType::External,
                        entry_point_selector: selector!("from_proxy"),
                        calldata: array![4],
                        storage_address: test_address(),
                        caller_address: 0.try_into().unwrap(),
                        call_type: CallType::Delegate,
                    },
                ];
            
                assert(trace == expected_trace, 'checker libcall from_proxy');
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

    assert_passed!(result);
}

#[test]
#[ignore]
fn trace_failed_library_call_from_test() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{declare, ContractClassTrait, test_address};
            use snforge_std::trace::{CallEntryPoint, CallType, EntryPointType, get_last_call_trace};
            
            use starknet::{ContractAddress, ClassHash};
            
            #[starknet::interface]
            trait ITraceInfoProxy<T> {
                fn with_libcall(ref self: T, class_hash: ClassHash) -> felt252;
                fn regular_call(self: @T, contract_address: ContractAddress) -> felt252;
                fn with_panic(self: @T, contract_address: ContractAddress);
            }
            
            #[starknet::interface]
            trait ITraceInfoChecker<T> {
                fn from_proxy(self: @T, data: felt252) -> felt252;
                fn panic(self: @T);
            }
            
            #[test]
            fn test_failed_library_call_trace_info() {
                let proxy_hash = declare('TraceInfoProxy').class_hash;
                let checker = declare('TraceInfoChecker');
                let checker_address = checker.deploy(@array![]).unwrap();
            
                let proxy_lib_dispatcher = ITraceInfoProxySafeLibraryDispatcher { class_hash: proxy_hash };
            
                match proxy_lib_dispatcher.with_panic(checker_address) {
                    Result::Ok(_) => panic_with_felt252('shouldve panicked'),
                    Result::Err(panic_data) => {
                        assert(*panic_data.at(0) == 'panic', *panic_data.at(0));
                        assert_trace_after_proxy_with_panic_call(get_last_call_trace(), checker_address);
                    }
                }
            
                let chcecker_lib_dispatcher = ITraceInfoCheckerSafeLibraryDispatcher {
                    class_hash: checker.class_hash
                };
            
                match chcecker_lib_dispatcher.panic() {
                    Result::Ok(_) => panic_with_felt252('shouldve panicked'),
                    Result::Err(panic_data) => {
                        assert(*panic_data.at(0) == 'panic', *panic_data.at(0));
                        assert_trace_after_checker_panic_call_from_test(get_last_call_trace(), checker_address);
                    }
                }
            }
            
            fn assert_trace_after_proxy_with_panic_call(
                trace: Array<CallEntryPoint>, checker_address: ContractAddress
            ) {
                let expected_trace = array![
                    CallEntryPoint {
                        entry_point_type: EntryPointType::External,
                        entry_point_selector: selector!("with_panic"),
                        calldata: array![checker_address.into()],
                        storage_address: test_address(),
                        caller_address: 0.try_into().unwrap(),
                        call_type: CallType::Delegate,
                    },
                    CallEntryPoint {
                        entry_point_type: EntryPointType::External,
                        entry_point_selector: selector!("panic"),
                        calldata: array![],
                        storage_address: checker_address,
                        caller_address: test_address(),
                        call_type: CallType::Call,
                    },
                ];
            
                assert(trace == expected_trace, 'proxy libcall with_panic');
            }
            
            fn assert_trace_after_checker_panic_call_from_test(
                trace: Array<CallEntryPoint>, checker_address: ContractAddress
            ) {
                let expected_trace = array![
                    CallEntryPoint {
                        entry_point_type: EntryPointType::External,
                        entry_point_selector: selector!("panic"),
                        calldata: array![],
                        storage_address: test_address(),
                        caller_address: 0.try_into().unwrap(),
                        call_type: CallType::Delegate,
                    },
                ];
            
                assert(trace == expected_trace, 'checker libcall panic');
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

    assert_passed!(result);
}

#[test]
fn trace_l1_handler() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{declare, ContractClassTrait, test_address, L1HandlerTrait,};
            use snforge_std::trace::{CallEntryPoint, CallType, EntryPointType, get_last_call_trace};
            
            use starknet::ContractAddress;
            
            #[test]
            fn test_l1_handler_call_trace_info() {
                let proxy = declare('TraceInfoProxy');
                let checker = declare('TraceInfoChecker');
            
                let checker_address = checker.deploy(@array![]).unwrap();
                let proxy_address = proxy.deploy(@array![checker_address.into()]).unwrap();
            
                let mut l1_handler = L1HandlerTrait::new(checker_address, function_name: 'handle_l1_message');
            
                l1_handler.from_address = 123;
                l1_handler.payload = array![proxy_address.into()].span();
            
                l1_handler.execute().unwrap();
                assert_trace_after_l1_handler_call(get_last_call_trace(), proxy_address, checker_address);
            }
            
            fn assert_trace_after_l1_handler_call(
                trace: Array<CallEntryPoint>, proxy_address: ContractAddress, checker_address: ContractAddress
            ) {
                let expected_trace = array![
                    CallEntryPoint {
                        entry_point_type: EntryPointType::L1Handler,
                        entry_point_selector: selector!("handle_l1_message"),
                        calldata: array![123, proxy_address.into()],
                        storage_address: checker_address,
                        caller_address: test_address(),
                        call_type: CallType::Call,
                    },
                    CallEntryPoint {
                        entry_point_type: EntryPointType::External,
                        entry_point_selector: selector!("regular_call"),
                        calldata: array![checker_address.into()],
                        storage_address: proxy_address,
                        caller_address: checker_address,
                        call_type: CallType::Call,
                    },
                    CallEntryPoint {
                        entry_point_type: EntryPointType::External,
                        entry_point_selector: selector!("from_proxy"),
                        calldata: array![2],
                        storage_address: checker_address,
                        caller_address: proxy_address,
                        call_type: CallType::Call,
                    },
                ];
            
                assert(trace == expected_trace, 'checker l1 handler');
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

    assert_passed!(result);
}
