use super::common::runner::{setup_package, test_runner};
use indoc::{formatdoc, indoc};
use shared::test_utils::output_assert::assert_stdout_contains;

#[test]
fn debugging_trace_custom_components() {
    let temp = setup_package("debugging");

    let output = test_runner(&temp)
        .arg("--trace-components")
        .arg("contract-name")
        .arg("call-result")
        .arg("call-type")
        .assert()
        .code(1);

    assert_stdout_contains(
        output,
        test_output(custom_output_trace_message, "debugging"),
    );
}

#[test]
fn debugging_trace_detailed() {
    let temp = setup_package("debugging");

    let output = test_runner(&temp)
        .arg("--trace-verbosity")
        .arg("detailed")
        .assert()
        .code(1);

    assert_stdout_contains(
        output,
        test_output(detailed_debugging_trace_message, "debugging"),
    );
}

#[test]
fn debugging_trace_detailed_fork() {
    let temp = setup_package("debugging_fork");

    let output = test_runner(&temp)
        .arg("--trace-verbosity")
        .arg("detailed")
        .assert()
        .code(1);

    assert_stdout_contains(
        output,
        test_output(detailed_debugging_trace_message_fork, "debugging_fork"),
    );
}

#[test]
fn debugging_trace_standard() {
    let temp = setup_package("debugging");

    let output = test_runner(&temp)
        .arg("--trace-verbosity")
        .arg("standard")
        .assert()
        .code(1);

    assert_stdout_contains(
        output,
        test_output(standard_debugging_trace_message, "debugging"),
    );
}

#[test]
fn debugging_trace_standard_fork() {
    let temp = setup_package("debugging_fork");

    let output = test_runner(&temp)
        .arg("--trace-verbosity")
        .arg("standard")
        .assert()
        .code(1);

    assert_stdout_contains(
        output,
        test_output(standard_debugging_trace_message_fork, "debugging_fork"),
    );
}

#[test]
fn debugging_trace_minimal() {
    let temp = setup_package("debugging");

    let output = test_runner(&temp)
        .arg("--trace-verbosity")
        .arg("minimal")
        .assert()
        .code(1);

    assert_stdout_contains(
        output,
        test_output(minimal_debugging_trace_message, "debugging"),
    );
}

#[test]
fn debugging_trace_minimal_fork() {
    let temp = setup_package("debugging_fork");

    let output = test_runner(&temp)
        .arg("--trace-verbosity")
        .arg("minimal")
        .assert()
        .code(1);

    assert_stdout_contains(
        output,
        test_output(minimal_debugging_trace_message_fork, "debugging_fork"),
    );
}

#[test]
fn debugging_double_flags() {
    let temp = setup_package("debugging");

    test_runner(&temp)
        .arg("--trace-verbosity")
        .arg("minimal")
        .arg("--trace-components")
        .arg("contract-name")
        .assert()
        .code(2)
        .stderr_eq(indoc! {"
            error: the argument '--trace-verbosity <TRACE_VERBOSITY>' cannot be used with '--trace-components <TRACE_COMPONENTS>...'

            Usage: snforge test --trace-verbosity <TRACE_VERBOSITY> [TEST_FILTER] [-- <ADDITIONAL_ARGS>...]

            For more information, try '--help'.
        "});
}

fn test_output(trace_message_fn: fn(&str, &str) -> String, package_name: &str) -> String {
    formatdoc! {r"
        [..]Compiling[..]
        [..]Finished[..]

        Collected 2 test(s) from {package_name} package
        Running 2 test(s) from tests/
        [FAIL] {package_name}_integrationtest::test_trace::test_debugging_trace_failure
        Failure data:
            (0x1, 0x2, 0x3, 0x4, 0x5, 0x454e545259504f494e545f4641494c4544 ('ENTRYPOINT_FAILED'))

        note: run with `SNFORGE_BACKTRACE=1` environment variable to display a backtrace
        {debugging_trace_fail}

        [PASS] {package_name}_integrationtest::test_trace::test_debugging_trace_success (l1_gas: ~[..], l1_data_gas: ~[..], l2_gas: ~[..])
        {debugging_trace_pass}

        Running 0 test(s) from src/
        Tests: 1 passed, 1 failed, 0 ignored, 0 filtered out

        Failures:
            {package_name}_integrationtest::test_trace::test_debugging_trace_failure
        ",
        debugging_trace_fail = trace_message_fn("failure", package_name),
        debugging_trace_pass = trace_message_fn("success", package_name)
    }
}

fn detailed_debugging_trace_message(test_name: &str, package_name: &str) -> String {
    formatdoc! {r"
        [test name] {package_name}_integrationtest::test_trace::test_debugging_trace_{test_name}
        ├─ [selector] execute_calls
        │  ├─ [contract name] SimpleContract
        │  ├─ [entry point type] External
        │  ├─ [calldata] array![RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}, RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}] }}, RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}]
        │  ├─ [contract address] [..]
        │  ├─ [caller address] [..]
        │  ├─ [call type] Call
        │  ├─ [call result] success: array![RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}, RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}] }}, RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}]
        │  ├─ [L2 gas] [..]
        │  ├─ [selector] execute_calls
        │  │  ├─ [contract name] SimpleContract
        │  │  ├─ [entry point type] External
        │  │  ├─ [calldata] array![RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}, RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}]
        │  │  ├─ [contract address] [..]
        │  │  ├─ [caller address] [..]
        │  │  ├─ [call type] Call
        │  │  ├─ [call result] success: array![RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}, RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}]
        │  │  ├─ [L2 gas] [..]
        │  │  ├─ [selector] execute_calls
        │  │  │  ├─ [contract name] SimpleContract
        │  │  │  ├─ [entry point type] External
        │  │  │  ├─ [calldata] array![]
        │  │  │  ├─ [contract address] [..]
        │  │  │  ├─ [caller address] [..]
        │  │  │  ├─ [call type] Call
        │  │  │  ├─ [call result] success: array![]
        │  │  │  └─ [L2 gas] [..]
        │  │  └─ [selector] execute_calls
        │  │     ├─ [contract name] SimpleContract
        │  │     ├─ [entry point type] External
        │  │     ├─ [calldata] array![]
        │  │     ├─ [contract address] [..]
        │  │     ├─ [caller address] [..]
        │  │     ├─ [call type] Call
        │  │     ├─ [call result] success: array![]
        │  │     └─ [L2 gas] [..]
        │  └─ [selector] execute_calls
        │     ├─ [contract name] SimpleContract
        │     ├─ [entry point type] External
        │     ├─ [calldata] array![]
        │     ├─ [contract address] [..]
        │     ├─ [caller address] [..]
        │     ├─ [call type] Call
        │     ├─ [call result] success: array![]
        │     └─ [L2 gas] [..]
        └─ [selector] fail
           ├─ [contract name] SimpleContract
           ├─ [entry point type] External
           ├─ [calldata] array![0x1, 0x2, 0x3, 0x4, 0x5]
           ├─ [contract address] [..]
           ├─ [caller address] [..]
           ├─ [call type] Call
           ├─ [call result] panic: (0x1, 0x2, 0x3, 0x4, 0x5)
           └─ [L2 gas] [..]
        "}
}

fn detailed_debugging_trace_message_fork(test_name: &str, package_name: &str) -> String {
    formatdoc! {r"
        [test name] {package_name}_integrationtest::test_trace::test_debugging_trace_{test_name}
        ├─ [selector] execute_calls
        │  ├─ [contract name] forked contract
        │  ├─ [entry point type] External
        │  ├─ [calldata] array![RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}, RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}] }}, RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}]
        │  ├─ [contract address] [..]
        │  ├─ [caller address] [..]
        │  ├─ [call type] Call
        │  ├─ [call result] success: array![RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}, RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}] }}, RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}]
        │  ├─ [L2 gas] [..]
        │  ├─ [selector] execute_calls
        │  │  ├─ [contract name] forked contract
        │  │  ├─ [entry point type] External
        │  │  ├─ [calldata] array![RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}, RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}]
        │  │  ├─ [contract address] [..]
        │  │  ├─ [caller address] [..]
        │  │  ├─ [call type] Call
        │  │  ├─ [call result] success: array![RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}, RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}]
        │  │  ├─ [L2 gas] [..]
        │  │  ├─ [selector] execute_calls
        │  │  │  ├─ [contract name] forked contract
        │  │  │  ├─ [entry point type] External
        │  │  │  ├─ [calldata] array![]
        │  │  │  ├─ [contract address] [..]
        │  │  │  ├─ [caller address] [..]
        │  │  │  ├─ [call type] Call
        │  │  │  ├─ [call result] success: array![]
        │  │  │  └─ [L2 gas] [..]
        │  │  └─ [selector] execute_calls
        │  │     ├─ [contract name] forked contract
        │  │     ├─ [entry point type] External
        │  │     ├─ [calldata] array![]
        │  │     ├─ [contract address] [..]
        │  │     ├─ [caller address] [..]
        │  │     ├─ [call type] Call
        │  │     ├─ [call result] success: array![]
        │  │     └─ [L2 gas] [..]
        │  └─ [selector] execute_calls
        │     ├─ [contract name] forked contract
        │     ├─ [entry point type] External
        │     ├─ [calldata] array![]
        │     ├─ [contract address] [..]
        │     ├─ [caller address] [..]
        │     ├─ [call type] Call
        │     ├─ [call result] success: array![]
        │     └─ [L2 gas] [..]
        └─ [selector] fail
           ├─ [contract name] forked contract
           ├─ [entry point type] External
           ├─ [calldata] array![0x1, 0x2, 0x3, 0x4, 0x5]
           ├─ [contract address] [..]
           ├─ [caller address] [..]
           ├─ [call type] Call
           ├─ [call result] panic: (0x1, 0x2, 0x3, 0x4, 0x5)
           └─ [L2 gas] [..]
        "}
}

fn standard_debugging_trace_message(test_name: &str, package_name: &str) -> String {
    formatdoc! {r"
        [test name] {package_name}_integrationtest::test_trace::test_debugging_trace_{test_name}
        ├─ [selector] execute_calls
        │  ├─ [contract name] SimpleContract
        │  ├─ [calldata] array![RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}, RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}] }}, RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}]
        │  ├─ [call result] success: array![RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}, RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}] }}, RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}]
        │  ├─ [selector] execute_calls
        │  │  ├─ [contract name] SimpleContract
        │  │  ├─ [calldata] array![RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}, RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}]
        │  │  ├─ [call result] success: array![RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}, RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}]
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

fn standard_debugging_trace_message_fork(test_name: &str, package_name: &str) -> String {
    formatdoc! {r"
        [test name] {package_name}_integrationtest::test_trace::test_debugging_trace_{test_name}
        ├─ [selector] execute_calls
        │  ├─ [contract name] forked contract
        │  ├─ [calldata] array![RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}, RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}] }}, RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}]
        │  ├─ [call result] success: array![RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}, RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}] }}, RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}]
        │  ├─ [selector] execute_calls
        │  │  ├─ [contract name] forked contract
        │  │  ├─ [calldata] array![RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}, RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}]
        │  │  ├─ [call result] success: array![RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}, RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}]
        │  │  ├─ [selector] execute_calls
        │  │  │  ├─ [contract name] forked contract
        │  │  │  ├─ [calldata] array![]
        │  │  │  └─ [call result] success: array![]
        │  │  └─ [selector] execute_calls
        │  │     ├─ [contract name] forked contract
        │  │     ├─ [calldata] array![]
        │  │     └─ [call result] success: array![]
        │  └─ [selector] execute_calls
        │     ├─ [contract name] forked contract
        │     ├─ [calldata] array![]
        │     └─ [call result] success: array![]
        └─ [selector] fail
           ├─ [contract name] forked contract
           ├─ [calldata] array![0x1, 0x2, 0x3, 0x4, 0x5]
           └─ [call result] panic: (0x1, 0x2, 0x3, 0x4, 0x5)
        "}
}

fn minimal_debugging_trace_message(test_name: &str, package_name: &str) -> String {
    formatdoc! {r"
        [test name] {package_name}_integrationtest::test_trace::test_debugging_trace_{test_name}
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

fn minimal_debugging_trace_message_fork(test_name: &str, package_name: &str) -> String {
    formatdoc! {r"
        [test name] {package_name}_integrationtest::test_trace::test_debugging_trace_{test_name}
        ├─ [selector] execute_calls
        │  ├─ [contract name] forked contract
        │  ├─ [selector] execute_calls
        │  │  ├─ [contract name] forked contract
        │  │  ├─ [selector] execute_calls
        │  │  │  └─ [contract name] forked contract
        │  │  └─ [selector] execute_calls
        │  │     └─ [contract name] forked contract
        │  └─ [selector] execute_calls
        │     └─ [contract name] forked contract
        └─ [selector] fail
           └─ [contract name] forked contract
        "}
}

fn custom_output_trace_message(test_name: &str, package_name: &str) -> String {
    formatdoc! {r"
        [test name] {package_name}_integrationtest::test_trace::test_debugging_trace_{test_name}
        ├─ [selector] execute_calls
        │  ├─ [contract name] SimpleContract
        │  ├─ [call type] Call
        │  ├─ [call result] success: array![RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}, RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}] }}, RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}]
        │  ├─ [selector] execute_calls
        │  │  ├─ [contract name] SimpleContract
        │  │  ├─ [call type] Call
        │  │  ├─ [call result] success: array![RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}, RecursiveCall {{ contract_address: ContractAddress([..]), payload: array![] }}]
        │  │  ├─ [selector] execute_calls
        │  │  │  ├─ [contract name] SimpleContract
        │  │  │  ├─ [call type] Call
        │  │  │  └─ [call result] success: array![]
        │  │  └─ [selector] execute_calls
        │  │     ├─ [contract name] SimpleContract
        │  │     ├─ [call type] Call
        │  │     └─ [call result] success: array![]
        │  └─ [selector] execute_calls
        │     ├─ [contract name] SimpleContract
        │     ├─ [call type] Call
        │     └─ [call result] success: array![]
        └─ [selector] fail
           ├─ [contract name] SimpleContract
           ├─ [call type] Call
           └─ [call result] panic: (0x1, 0x2, 0x3, 0x4, 0x5)
        "}
}
