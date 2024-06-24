use snforge_std::{declare, ContractClassTrait};

use trace_resources::{
    trace_info_checker::{
        ITraceInfoCheckerSafeLibraryDispatcher, ITraceInfoCheckerSafeDispatcherTrait
    },
    trace_info_proxy::{ITraceInfoProxySafeLibraryDispatcher, ITraceInfoProxySafeDispatcherTrait,}
};

#[test]
#[feature("safe_dispatcher")]
fn test_failed_lib_call() {
    let empty_hash = declare("Empty").unwrap().class_hash;
    let proxy_hash = declare("TraceInfoProxy").unwrap().class_hash;
    let checker = declare("TraceInfoChecker").unwrap();
    let (checker_address, _) = checker.deploy(@array![]).unwrap();

    trace_resources::use_builtins_and_syscalls(empty_hash, 7);

    let proxy_lib_dispatcher = ITraceInfoProxySafeLibraryDispatcher { class_hash: proxy_hash };
    match proxy_lib_dispatcher.with_panic(checker_address, empty_hash, 1) {
        Result::Ok(_) => panic_with_felt252('shouldve panicked'),
        Result::Err(panic_data) => { assert(*panic_data.at(0) == 'panic', *panic_data.at(0)); }
    }

    let chcecker_lib_dispatcher = ITraceInfoCheckerSafeLibraryDispatcher {
        class_hash: checker.class_hash
    };
    match chcecker_lib_dispatcher.panic(empty_hash, 2) {
        Result::Ok(_) => panic_with_felt252('shouldve panicked'),
        Result::Err(panic_data) => { assert(*panic_data.at(0) == 'panic', *panic_data.at(0)); }
    };
}
