use snforge_std::cheatcodes::contract_class::DeclareResultTrait;
use snforge_std::trace::get_call_trace;
use snforge_std::{ContractClassTrait, declare};
use trace_info::{
    FailingDispatcher, FailingDispatcherTrait, RecursiveCall, RecursiveCallerDispatcher,
    RecursiveCallerDispatcherTrait,
};

#[test]
#[should_panic]
fn test_debugging_trace_success() {
    run_test();
}

#[test]
fn test_debugging_trace_failure() {
    run_test();
}

fn run_test() {
    let sc = declare("SimpleContract").unwrap().contract_class();

    let (contract_address_A, _) = sc.deploy(@array![]).unwrap();
    let (contract_address_B, _) = sc.deploy(@array![]).unwrap();
    let (contract_address_C, _) = sc.deploy(@array![]).unwrap();

    let calls = array![
        RecursiveCall {
            contract_address: contract_address_B,
            payload: array![
                RecursiveCall { contract_address: contract_address_C, payload: array![] },
                RecursiveCall { contract_address: contract_address_C, payload: array![] },
            ],
        },
        RecursiveCall { contract_address: contract_address_C, payload: array![] },
    ];

    RecursiveCallerDispatcher { contract_address: contract_address_A }.execute_calls(calls);

    let failing_dispatcher = FailingDispatcher { contract_address: contract_address_A };
    failing_dispatcher.fail(array![1, 2, 3, 4, 5]);
}
