use super::common::runner::{setup_package, test_runner};
use indoc::formatdoc;
use shared::test_utils::output_assert::assert_stdout_contains;

#[test]
fn debugging_trace() {
    let temp = setup_package("debugging");

    let output = test_runner(&temp).assert().code(1);

    assert_stdout_contains(
        output,
        formatdoc! {r"
        [..]Compiling[..]
        [..]Finished[..]

        Collected 2 test(s) from trace_info package
        Running 2 test(s) from tests/
        [FAIL] trace_info_integrationtest::test_trace::test_debugging_trace_fail
        Failure data:
            (0x1, 0x2, 0x3, 0x4, 0x5)

        note: run with `SNFORGE_BACKTRACE=1` environment variable to display a backtrace
        {debugging_trace_fail}

        [PASS] trace_info_integrationtest::test_trace::test_debugging_trace_success (l1_gas: ~0, l1_data_gas: ~288, l2_gas: ~1440000)
        {debugging_trace_pass}

        Running 0 test(s) from src/
        Tests: 1 passed, 1 failed, 0 skipped, 0 ignored, 0 filtered out

        Failures:
            trace_info_integrationtest::test_trace::test_debugging_trace_fail
        ",
        debugging_trace_fail = debugging_trace_message("fail"),
        debugging_trace_pass = debugging_trace_message("success")},
    );
}

fn debugging_trace_message(test_name: &str) -> String {
    formatdoc! {r"
        [test name] trace_info_integrationtest::test_trace::test_debugging_trace_{test_name}
        ├─ [selector] execute_calls
        │  ├─ [contract name] SimpleContract
        │  ├─ [entry point type] External
        │  ├─ [calldata] array![RecursiveCall {{ contract_address: ContractAddress(0x634cf632813aca745d024ee244aab954461a7341b610b103fa7569bf1e14a5e), payload: array![RecursiveCall {{ contract_address: ContractAddress(0x38767c97f072a291507aa962d6d92b04ae7b4e01c406717f5485b7a86fbdde7), payload: array![] }}, RecursiveCall {{ contract_address: ContractAddress(0x38767c97f072a291507aa962d6d92b04ae7b4e01c406717f5485b7a86fbdde7), payload: array![] }}] }}, RecursiveCall {{ contract_address: ContractAddress(0x38767c97f072a291507aa962d6d92b04ae7b4e01c406717f5485b7a86fbdde7), payload: array![] }}]
        │  ├─ [storage address] 0x5ab84515ea91a99261961ba72888d663606612958a363e8538302a27398bc29
        │  ├─ [caller address] 0x1724987234973219347210837402
        │  ├─ [call type] Call
        │  ├─ [call result] success: []
        │  ├─ [selector] execute_calls
        │  │  ├─ [contract name] SimpleContract
        │  │  ├─ [entry point type] External
        │  │  ├─ [calldata] array![RecursiveCall {{ contract_address: ContractAddress(0x38767c97f072a291507aa962d6d92b04ae7b4e01c406717f5485b7a86fbdde7), payload: array![] }}, RecursiveCall {{ contract_address: ContractAddress(0x38767c97f072a291507aa962d6d92b04ae7b4e01c406717f5485b7a86fbdde7), payload: array![] }}]
        │  │  ├─ [storage address] 0x634cf632813aca745d024ee244aab954461a7341b610b103fa7569bf1e14a5e
        │  │  ├─ [caller address] 0x5ab84515ea91a99261961ba72888d663606612958a363e8538302a27398bc29
        │  │  ├─ [call type] Call
        │  │  ├─ [call result] success: []
        │  │  ├─ [selector] execute_calls
        │  │  │  ├─ [contract name] SimpleContract
        │  │  │  ├─ [entry point type] External
        │  │  │  ├─ [calldata] array![]
        │  │  │  ├─ [storage address] 0x38767c97f072a291507aa962d6d92b04ae7b4e01c406717f5485b7a86fbdde7
        │  │  │  ├─ [caller address] 0x634cf632813aca745d024ee244aab954461a7341b610b103fa7569bf1e14a5e
        │  │  │  ├─ [call type] Call
        │  │  │  └─ [call result] success: []
        │  │  └─ [selector] execute_calls
        │  │     ├─ [contract name] SimpleContract
        │  │     ├─ [entry point type] External
        │  │     ├─ [calldata] array![]
        │  │     ├─ [storage address] 0x38767c97f072a291507aa962d6d92b04ae7b4e01c406717f5485b7a86fbdde7
        │  │     ├─ [caller address] 0x634cf632813aca745d024ee244aab954461a7341b610b103fa7569bf1e14a5e
        │  │     ├─ [call type] Call
        │  │     └─ [call result] success: []
        │  └─ [selector] execute_calls
        │     ├─ [contract name] SimpleContract
        │     ├─ [entry point type] External
        │     ├─ [calldata] array![]
        │     ├─ [storage address] 0x38767c97f072a291507aa962d6d92b04ae7b4e01c406717f5485b7a86fbdde7
        │     ├─ [caller address] 0x5ab84515ea91a99261961ba72888d663606612958a363e8538302a27398bc29
        │     ├─ [call type] Call
        │     └─ [call result] success: []
        └─ [selector] fail
           ├─ [contract name] SimpleContract
           ├─ [entry point type] External
           ├─ [calldata] array![0x1, 0x2, 0x3, 0x4, 0x5]
           ├─ [storage address] 0x5ab84515ea91a99261961ba72888d663606612958a363e8538302a27398bc29
           ├─ [caller address] 0x1724987234973219347210837402
           ├─ [call type] Call
           └─ [call result] panic: [0x1, 0x2, 0x3, 0x4, 0x5]
        "}
}
