use core::panic_with_felt252;
use snforge_std::cheatcodes::contract_class::DeclareResultTrait;
use snforge_std::trace::get_call_trace;
use snforge_std::{ContractClassTrait, declare};
use trace_info::{
    FailingSafeDispatcher, FailingSafeDispatcherTrait, RecursiveCall, RecursiveCallerDispatcher,
    RecursiveCallerDispatcherTrait,
};

#[test]
#[feature("safe_dispatcher")]
fn test_trace() {
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

    let failing_dispatcher = FailingSafeDispatcher { contract_address: contract_address_A };
    match failing_dispatcher.fail(array![1, 2, 3, 4, 5]) {
        Result::Ok(_) => panic_with_felt252('shouldve panicked'),
        Result::Err(panic_data) => { assert(panic_data == array![1, 2, 3, 4, 5], ''); },
    }

    println!("{}", get_call_trace());
}
