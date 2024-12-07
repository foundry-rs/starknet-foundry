use core::clone::Clone;
use snforge_std::cheatcodes::contract_class::DeclareResultTrait;
use snforge_std::{declare, ContractClassTrait};


use trace_resources::{
    trace_info_checker::{ITraceInfoCheckerLibraryDispatcher, ITraceInfoCheckerDispatcherTrait},
    trace_info_proxy::{ITraceInfoProxyLibraryDispatcher, ITraceInfoProxyDispatcherTrait,}
};

#[test]
fn test_lib_call() {
    let empty_hash = declare("Empty").unwrap().contract_class().class_hash.clone();
    let proxy_hash = declare("TraceInfoProxy").unwrap().contract_class().class_hash.clone();
    let checker = declare("TraceInfoChecker").unwrap().contract_class().clone();
    let dummy = declare("TraceDummy").unwrap().contract_class();

    trace_resources::use_builtins_and_syscalls(empty_hash, 7);

    let (checker_address, _) = checker.deploy(@array![]).unwrap();
    let (dummy_address, _) = dummy.deploy(@array![]).unwrap();

    let proxy_lib_dispatcher = ITraceInfoProxyLibraryDispatcher { class_hash: proxy_hash };

    proxy_lib_dispatcher.regular_call(checker_address, empty_hash, 1);
    proxy_lib_dispatcher.with_libcall(checker.class_hash, empty_hash, 2);
    proxy_lib_dispatcher.call_two(checker_address, dummy_address, empty_hash, 3);

    let chcecker_lib_dispatcher = ITraceInfoCheckerLibraryDispatcher {
        class_hash: checker.class_hash
    };

    chcecker_lib_dispatcher.from_proxy(4, empty_hash, 4);
}
