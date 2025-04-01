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
        Success data:
            (0x1, 0x2, 0x3, 0x4, 0x5)

        note: run with `SNFORGE_BACKTRACE=1` environment variable to display a backtrace
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
        │  ├─ [calldata] [0x2, 0x73668021e0dfc00aa573e654a0763dca79dbbfb5a60e4987f71a3472026994a, 0x2, 0x6ce4a1209f067bb71fbd98f37e1240f45beaf8fb22c39628cb2accfe4fb5bcb, 0x0, 0x6ce4a1209f067bb71fbd98f37e1240f45beaf8fb22c39628cb2accfe4fb5bcb, 0x0, 0x6ce4a1209f067bb71fbd98f37e1240f45beaf8fb22c39628cb2accfe4fb5bcb, 0x0]
        │  ├─ [storage address] 0x29e61a26d304881d5944a9ab3446685cb791af1ff827f9d4bc34489a2ff9629
        │  ├─ [caller address] 0x1724987234973219347210837402
        │  ├─ [call type] Call
        │  ├─ [call result] success: []
        │  ├─ [selector] execute_calls
        │  │  ├─ [contract name] SimpleContract
        │  │  ├─ [entry point type] External
        │  │  ├─ [calldata] [0x2, 0x6ce4a1209f067bb71fbd98f37e1240f45beaf8fb22c39628cb2accfe4fb5bcb, 0x0, 0x6ce4a1209f067bb71fbd98f37e1240f45beaf8fb22c39628cb2accfe4fb5bcb, 0x0]
        │  │  ├─ [storage address] 0x73668021e0dfc00aa573e654a0763dca79dbbfb5a60e4987f71a3472026994a
        │  │  ├─ [caller address] 0x29e61a26d304881d5944a9ab3446685cb791af1ff827f9d4bc34489a2ff9629
        │  │  ├─ [call type] Call
        │  │  ├─ [call result] success: []
        │  │  ├─ [selector] execute_calls
        │  │  │  ├─ [contract name] SimpleContract
        │  │  │  ├─ [entry point type] External
        │  │  │  ├─ [calldata] [0x0]
        │  │  │  ├─ [storage address] 0x6ce4a1209f067bb71fbd98f37e1240f45beaf8fb22c39628cb2accfe4fb5bcb
        │  │  │  ├─ [caller address] 0x73668021e0dfc00aa573e654a0763dca79dbbfb5a60e4987f71a3472026994a
        │  │  │  ├─ [call type] Call
        │  │  │  └─ [call result] success: []
        │  │  └─ [selector] execute_calls
        │  │     ├─ [contract name] SimpleContract
        │  │     ├─ [entry point type] External
        │  │     ├─ [calldata] [0x0]
        │  │     ├─ [storage address] 0x6ce4a1209f067bb71fbd98f37e1240f45beaf8fb22c39628cb2accfe4fb5bcb
        │  │     ├─ [caller address] 0x73668021e0dfc00aa573e654a0763dca79dbbfb5a60e4987f71a3472026994a
        │  │     ├─ [call type] Call
        │  │     └─ [call result] success: []
        │  └─ [selector] execute_calls
        │     ├─ [contract name] SimpleContract
        │     ├─ [entry point type] External
        │     ├─ [calldata] [0x0]
        │     ├─ [storage address] 0x6ce4a1209f067bb71fbd98f37e1240f45beaf8fb22c39628cb2accfe4fb5bcb
        │     ├─ [caller address] 0x29e61a26d304881d5944a9ab3446685cb791af1ff827f9d4bc34489a2ff9629
        │     ├─ [call type] Call
        │     └─ [call result] success: []
        └─ [selector] fail
           ├─ [contract name] SimpleContract
           ├─ [entry point type] External
           ├─ [calldata] [0x5, 0x1, 0x2, 0x3, 0x4, 0x5]
           ├─ [storage address] 0x29e61a26d304881d5944a9ab3446685cb791af1ff827f9d4bc34489a2ff9629
           ├─ [caller address] 0x1724987234973219347210837402
           ├─ [call type] Call
           └─ [call result] panic: [0x1, 0x2, 0x3, 0x4, 0x5]
        "}
}
