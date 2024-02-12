use snforge_std::{declare, ContractClassTrait};
use snforge_std::trace::{get_call_trace};

use trace_info::{RecursiveCallerDispatcher, RecursiveCallerDispatcherTrait, RecursiveCall};

#[test]
fn test_trace_print() {
    let sc = declare('SimpleContract');

    let contract_address_A = sc.deploy(@array![]).unwrap();
    let contract_address_B = sc.deploy(@array![]).unwrap();
    let contract_address_C = sc.deploy(@array![]).unwrap();

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

    println!("{}", get_call_trace());
}
