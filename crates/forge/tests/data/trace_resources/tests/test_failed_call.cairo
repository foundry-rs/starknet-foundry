use snforge_std::{declare, ContractClassTrait};

use trace_resources::{
    trace_info_checker::{ITraceInfoCheckerSafeDispatcherTrait, ITraceInfoCheckerSafeDispatcher},
    trace_info_proxy::{ITraceInfoProxySafeDispatcherTrait, ITraceInfoProxySafeDispatcher,}
};

#[test]
fn test_failed_call() {
    let empty_hash = declare("Empty").class_hash;
    let proxy = declare("TraceInfoProxy");
    let checker = declare("TraceInfoChecker");

    trace_resources::use_builtins_and_syscalls(empty_hash, 7);

    let checker_address = checker.deploy(@array![]).unwrap();
    let proxy_address = proxy
        .deploy(@array![checker_address.into(), empty_hash.into(), 1])
        .unwrap();

    let proxy_dispatcher = ITraceInfoProxySafeDispatcher { contract_address: proxy_address };

    #[feature("safe_dispatcher")]
    match proxy_dispatcher.with_panic(checker_address, empty_hash, 2) {
        Result::Ok(_) => panic_with_felt252('shouldve panicked'),
        Result::Err(panic_data) => { assert(*panic_data.at(0) == 'panic', *panic_data.at(0)); }
    }

    let chcecker_dispatcher = ITraceInfoCheckerSafeDispatcher { contract_address: checker_address };

    #[feature("safe_dispatcher")]
    match chcecker_dispatcher.panic(empty_hash, 3) {
        Result::Ok(_) => panic_with_felt252('shouldve panicked'),
        Result::Err(panic_data) => { assert(*panic_data.at(0) == 'panic', *panic_data.at(0)); }
    };
}
