use super::common::runner::{setup_package, test_runner};
use indoc::formatdoc;
use shared::test_utils::output_assert::assert_stdout_contains;

#[test]
fn debugging_trace_detailed() {
    let temp = setup_package("debugging");

    let output = test_runner(&temp)
        .arg("--trace-verbosity")
        .arg("detailed")
        .assert()
        .code(1);

    assert_stdout_contains(output, test_output(detailed_debugging_trace_message));
}

#[test]
fn debugging_trace_standard() {
    let temp = setup_package("debugging");

    let output = test_runner(&temp)
        .arg("--trace-verbosity")
        .arg("standard")
        .assert()
        .code(1);

    assert_stdout_contains(output, test_output(standard_debugging_trace_message));
}

#[test]
fn debugging_trace_minimal() {
    let temp = setup_package("debugging");

    let output = test_runner(&temp)
        .arg("--trace-verbosity")
        .arg("minimal")
        .assert()
        .code(1);

    assert_stdout_contains(output, test_output(minimal_debugging_trace_message));
}

fn test_output(trace_message_fn: fn(&str) -> String) -> String {
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

        [PASS] trace_info_integrationtest::test_trace::test_debugging_trace_success (l1_gas: ~0, l1_data_gas: ~288, l2_gas: ~1600000)
        {debugging_trace_pass}

        Running 0 test(s) from src/
        Tests: 1 passed, 1 failed, 0 skipped, 0 ignored, 0 filtered out

        Failures:
            trace_info_integrationtest::test_trace::test_debugging_trace_fail
        ",
        debugging_trace_fail = trace_message_fn("fail"),
        debugging_trace_pass = trace_message_fn("success")
    }
}

fn detailed_debugging_trace_message(test_name: &str) -> String {
    formatdoc! {r"
        [test name] trace_info_integrationtest::test_trace::test_debugging_trace_{test_name}
        ├─ [selector] execute_calls
        │  ├─ [contract name] SimpleContract
        │  ├─ [entry point type] External
        │  ├─ [calldata] array![RecursiveCall {{ contract_address: ContractAddress(0x10a2fac439604ce4129fe7c205b711e8141e12e2e52e08f7f898fe7ac13f0a), payload: array![RecursiveCall {{ contract_address: ContractAddress(0x28f58bf524dc0adcf7468c67d7ffdac1e5d885d347c6a498978f538984dbda), payload: array![] }}, RecursiveCall {{ contract_address: ContractAddress(0x28f58bf524dc0adcf7468c67d7ffdac1e5d885d347c6a498978f538984dbda), payload: array![] }}] }}, RecursiveCall {{ contract_address: ContractAddress(0x28f58bf524dc0adcf7468c67d7ffdac1e5d885d347c6a498978f538984dbda), payload: array![] }}]
        │  ├─ [storage address] 0x7b29abec6baad44d169ee10b37c9a1eae834d71887607f60d2f90836f6eb973
        │  ├─ [caller address] 0x1724987234973219347210837402
        │  ├─ [call type] Call
        │  ├─ [call result] success: array![RecursiveCall {{ contract_address: ContractAddress(0x10a2fac439604ce4129fe7c205b711e8141e12e2e52e08f7f898fe7ac13f0a), payload: array![RecursiveCall {{ contract_address: ContractAddress(0x28f58bf524dc0adcf7468c67d7ffdac1e5d885d347c6a498978f538984dbda), payload: array![] }}, RecursiveCall {{ contract_address: ContractAddress(0x28f58bf524dc0adcf7468c67d7ffdac1e5d885d347c6a498978f538984dbda), payload: array![] }}] }}, RecursiveCall {{ contract_address: ContractAddress(0x28f58bf524dc0adcf7468c67d7ffdac1e5d885d347c6a498978f538984dbda), payload: array![] }}]
        │  ├─ [selector] execute_calls
        │  │  ├─ [contract name] SimpleContract
        │  │  ├─ [entry point type] External
        │  │  ├─ [calldata] array![RecursiveCall {{ contract_address: ContractAddress(0x28f58bf524dc0adcf7468c67d7ffdac1e5d885d347c6a498978f538984dbda), payload: array![] }}, RecursiveCall {{ contract_address: ContractAddress(0x28f58bf524dc0adcf7468c67d7ffdac1e5d885d347c6a498978f538984dbda), payload: array![] }}]
        │  │  ├─ [storage address] 0x10a2fac439604ce4129fe7c205b711e8141e12e2e52e08f7f898fe7ac13f0a
        │  │  ├─ [caller address] 0x7b29abec6baad44d169ee10b37c9a1eae834d71887607f60d2f90836f6eb973
        │  │  ├─ [call type] Call
        │  │  ├─ [call result] success: array![RecursiveCall {{ contract_address: ContractAddress(0x28f58bf524dc0adcf7468c67d7ffdac1e5d885d347c6a498978f538984dbda), payload: array![] }}, RecursiveCall {{ contract_address: ContractAddress(0x28f58bf524dc0adcf7468c67d7ffdac1e5d885d347c6a498978f538984dbda), payload: array![] }}]
        │  │  ├─ [selector] execute_calls
        │  │  │  ├─ [contract name] SimpleContract
        │  │  │  ├─ [entry point type] External
        │  │  │  ├─ [calldata] array![]
        │  │  │  ├─ [storage address] 0x28f58bf524dc0adcf7468c67d7ffdac1e5d885d347c6a498978f538984dbda
        │  │  │  ├─ [caller address] 0x10a2fac439604ce4129fe7c205b711e8141e12e2e52e08f7f898fe7ac13f0a
        │  │  │  ├─ [call type] Call
        │  │  │  └─ [call result] success: array![]
        │  │  └─ [selector] execute_calls
        │  │     ├─ [contract name] SimpleContract
        │  │     ├─ [entry point type] External
        │  │     ├─ [calldata] array![]
        │  │     ├─ [storage address] 0x28f58bf524dc0adcf7468c67d7ffdac1e5d885d347c6a498978f538984dbda
        │  │     ├─ [caller address] 0x10a2fac439604ce4129fe7c205b711e8141e12e2e52e08f7f898fe7ac13f0a
        │  │     ├─ [call type] Call
        │  │     └─ [call result] success: array![]
        │  └─ [selector] execute_calls
        │     ├─ [contract name] SimpleContract
        │     ├─ [entry point type] External
        │     ├─ [calldata] array![]
        │     ├─ [storage address] 0x28f58bf524dc0adcf7468c67d7ffdac1e5d885d347c6a498978f538984dbda
        │     ├─ [caller address] 0x7b29abec6baad44d169ee10b37c9a1eae834d71887607f60d2f90836f6eb973
        │     ├─ [call type] Call
        │     └─ [call result] success: array![]
        └─ [selector] fail
           ├─ [contract name] SimpleContract
           ├─ [entry point type] External
           ├─ [calldata] array![0x1, 0x2, 0x3, 0x4, 0x5]
           ├─ [storage address] 0x7b29abec6baad44d169ee10b37c9a1eae834d71887607f60d2f90836f6eb973
           ├─ [caller address] 0x1724987234973219347210837402
           ├─ [call type] Call
           └─ [call result] panic: (0x1, 0x2, 0x3, 0x4, 0x5)
        "}
}

fn standard_debugging_trace_message(test_name: &str) -> String {
    formatdoc! {r"
        [test name] trace_info_integrationtest::test_trace::test_debugging_trace_{test_name}
        ├─ [selector] execute_calls
        │  ├─ [contract name] SimpleContract
        │  ├─ [calldata] array![RecursiveCall {{ contract_address: ContractAddress(0x10a2fac439604ce4129fe7c205b711e8141e12e2e52e08f7f898fe7ac13f0a), payload: array![RecursiveCall {{ contract_address: ContractAddress(0x28f58bf524dc0adcf7468c67d7ffdac1e5d885d347c6a498978f538984dbda), payload: array![] }}, RecursiveCall {{ contract_address: ContractAddress(0x28f58bf524dc0adcf7468c67d7ffdac1e5d885d347c6a498978f538984dbda), payload: array![] }}] }}, RecursiveCall {{ contract_address: ContractAddress(0x28f58bf524dc0adcf7468c67d7ffdac1e5d885d347c6a498978f538984dbda), payload: array![] }}]
        │  ├─ [call result] success: array![RecursiveCall {{ contract_address: ContractAddress(0x10a2fac439604ce4129fe7c205b711e8141e12e2e52e08f7f898fe7ac13f0a), payload: array![RecursiveCall {{ contract_address: ContractAddress(0x28f58bf524dc0adcf7468c67d7ffdac1e5d885d347c6a498978f538984dbda), payload: array![] }}, RecursiveCall {{ contract_address: ContractAddress(0x28f58bf524dc0adcf7468c67d7ffdac1e5d885d347c6a498978f538984dbda), payload: array![] }}] }}, RecursiveCall {{ contract_address: ContractAddress(0x28f58bf524dc0adcf7468c67d7ffdac1e5d885d347c6a498978f538984dbda), payload: array![] }}]
        │  ├─ [selector] execute_calls
        │  │  ├─ [contract name] SimpleContract
        │  │  ├─ [calldata] array![RecursiveCall {{ contract_address: ContractAddress(0x28f58bf524dc0adcf7468c67d7ffdac1e5d885d347c6a498978f538984dbda), payload: array![] }}, RecursiveCall {{ contract_address: ContractAddress(0x28f58bf524dc0adcf7468c67d7ffdac1e5d885d347c6a498978f538984dbda), payload: array![] }}]
        │  │  ├─ [call result] success: array![RecursiveCall {{ contract_address: ContractAddress(0x28f58bf524dc0adcf7468c67d7ffdac1e5d885d347c6a498978f538984dbda), payload: array![] }}, RecursiveCall {{ contract_address: ContractAddress(0x28f58bf524dc0adcf7468c67d7ffdac1e5d885d347c6a498978f538984dbda), payload: array![] }}]
        │  │  ├─ [selector] execute_calls
        │  │  │  ├─ [contract name] SimpleContract
        │  │  │  ├─ [calldata] array![]
        │  │  │  └─ [call result] success: array![]
        │  │  └─ [selector] execute_calls
        │  │     ├─ [contract name] SimpleContract
        │  │     ├─ [calldata] array![]
        │  │     └─ [call result] success: array![]
        │  └─ [selector] execute_calls
        │     ├─ [contract name] SimpleContract
        │     ├─ [calldata] array![]
        │     └─ [call result] success: array![]
        └─ [selector] fail
           ├─ [contract name] SimpleContract
           ├─ [calldata] array![0x1, 0x2, 0x3, 0x4, 0x5]
           └─ [call result] panic: (0x1, 0x2, 0x3, 0x4, 0x5)
        "}
}

fn minimal_debugging_trace_message(test_name: &str) -> String {
    formatdoc! {r"
        [test name] trace_info_integrationtest::test_trace::test_debugging_trace_{test_name}
        ├─ [selector] execute_calls
        │  ├─ [contract name] SimpleContract
        │  ├─ [selector] execute_calls
        │  │  ├─ [contract name] SimpleContract
        │  │  ├─ [selector] execute_calls
        │  │  │  └─ [contract name] SimpleContract
        │  │  └─ [selector] execute_calls
        │  │     └─ [contract name] SimpleContract
        │  └─ [selector] execute_calls
        │     └─ [contract name] SimpleContract
        └─ [selector] fail
           └─ [contract name] SimpleContract
        "}
}
