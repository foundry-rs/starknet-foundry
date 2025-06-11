use snforge_std::cheatcodes::contract_class::DeclareResultTrait;
use snforge_std::{declare, ContractClassTrait};
use snforge_std::trace::{get_call_trace};

use trace_info::{
    RecursiveCallerDispatcher, RecursiveCallerDispatcherTrait, RecursiveCall, FailingDispatcher,
    FailingDispatcherTrait,
};

#[test]
#[should_panic]
#[fork(url: "{{ NODE_RPC_URL }}", block_number: 828912)]
fn test_debugging_trace_success() {
    run_test();
}

#[test]
#[fork(url: "{{ NODE_RPC_URL }}", block_number: 828912)]
fn test_debugging_trace_failure() {
    run_test();
}

fn run_test() {
    let contract_address_A = 0x05005956a18de174e33378fa9a278dde8a22d0bf823d4bd6f9c9051fe99d04a0
        .try_into()
        .unwrap();
    let contract_address_B = 0x01c6cec47a2ed95320f76e2b50626879737aa57990748db2d9527e867f98bc55
        .try_into()
        .unwrap();
    let contract_address_C = 0x042e631f2785a56bc5bcfd247b16e72d3d957bb8efe136829379a20ac675dbb7
        .try_into()
        .unwrap();

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
