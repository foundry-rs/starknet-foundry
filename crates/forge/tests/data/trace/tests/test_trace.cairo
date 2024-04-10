use snforge_std::{declare, ContractClassTrait};
use snforge_std::trace::{get_call_trace};

use trace_info::{
    RecursiveCallerDispatcher, RecursiveCallerDispatcherTrait, RecursiveCall, FailingSafeDispatcher,
    FailingSafeDispatcherTrait
};

#[test]
#[feature("safe_dispatcher")]
fn test_trace_print() {
    let sc = declare("SimpleContract").unwrap();

    let (contract_address_A, _) = sc.deploy(@array![]).unwrap();
    let (contract_address_B, _) = sc.deploy(@array![]).unwrap();
    let (contract_address_C, _) = sc.deploy(@array![]).unwrap();

    let calls = array![
        RecursiveCall {
            contract_address: contract_address_B,
            payload: array![
                RecursiveCall { contract_address: contract_address_C, payload: array![], },
                RecursiveCall { contract_address: contract_address_C, payload: array![], }
            ]
        },
        RecursiveCall { contract_address: contract_address_C, payload: array![], }
    ];

    RecursiveCallerDispatcher { contract_address: contract_address_A }.execute_calls(calls);

    let failing_dispatcher = FailingSafeDispatcher { contract_address: contract_address_A };
    match failing_dispatcher.fail(array![1, 2, 3, 4, 5]) {
        Result::Ok(_) => panic_with_felt252('shouldve panicked'),
        Result::Err(panic_data) => { assert(panic_data == array![1, 2, 3, 4, 5], ''); }
    }

    println!("{}", get_call_trace());
}
