use super::common::runner::{setup_package, test_runner};
use indoc::{formatdoc, indoc};
use shared::test_utils::output_assert::assert_stdout_contains;

const DEBUGGING_TRACE: &str = indoc! {r"
        [selector] 0x17340c6779204ea2a91c87d1c2226a3aebda65c64da3672a36893c4330ea27b
        ├─ [entry point type] External
        ├─ [calldata] []
        ├─ [storage address] 0x1724987234973219347210837402
        ├─ [caller address] 0x0
        ├─ [call type] Call
        ├─ [call result] success: []
        ├─ [selector] 0x387cbcd4e21cd976a8be7d24cd16b8e65e1940f9e06ded4f42798cf64c089be
        │  ├─ [entry point type] External
        │  ├─ [calldata] [0x2, 0x73668021e0dfc00aa573e654a0763dca79dbbfb5a60e4987f71a3472026994a, 0x2, 0x6ce4a1209f067bb71fbd98f37e1240f45beaf8fb22c39628cb2accfe4fb5bcb, 0x0, 0x6ce4a1209f067bb71fbd98f37e1240f45beaf8fb22c39628cb2accfe4fb5bcb, 0x0, 0x6ce4a1209f067bb71fbd98f37e1240f45beaf8fb22c39628cb2accfe4fb5bcb, 0x0]
        │  ├─ [storage address] 0x29e61a26d304881d5944a9ab3446685cb791af1ff827f9d4bc34489a2ff9629
        │  ├─ [caller address] 0x1724987234973219347210837402
        │  ├─ [call type] Call
        │  ├─ [call result] success: []
        │  ├─ [selector] 0x387cbcd4e21cd976a8be7d24cd16b8e65e1940f9e06ded4f42798cf64c089be
        │  │  ├─ [entry point type] External
        │  │  ├─ [calldata] [0x2, 0x6ce4a1209f067bb71fbd98f37e1240f45beaf8fb22c39628cb2accfe4fb5bcb, 0x0, 0x6ce4a1209f067bb71fbd98f37e1240f45beaf8fb22c39628cb2accfe4fb5bcb, 0x0]
        │  │  ├─ [storage address] 0x73668021e0dfc00aa573e654a0763dca79dbbfb5a60e4987f71a3472026994a
        │  │  ├─ [caller address] 0x29e61a26d304881d5944a9ab3446685cb791af1ff827f9d4bc34489a2ff9629
        │  │  ├─ [call type] Call
        │  │  ├─ [call result] success: []
        │  │  ├─ [selector] 0x387cbcd4e21cd976a8be7d24cd16b8e65e1940f9e06ded4f42798cf64c089be
        │  │  │  ├─ [entry point type] External
        │  │  │  ├─ [calldata] [0x0]
        │  │  │  ├─ [storage address] 0x6ce4a1209f067bb71fbd98f37e1240f45beaf8fb22c39628cb2accfe4fb5bcb
        │  │  │  ├─ [caller address] 0x73668021e0dfc00aa573e654a0763dca79dbbfb5a60e4987f71a3472026994a
        │  │  │  ├─ [call type] Call
        │  │  │  └─ [call result] success: []
        │  │  └─ [selector] 0x387cbcd4e21cd976a8be7d24cd16b8e65e1940f9e06ded4f42798cf64c089be
        │  │     ├─ [entry point type] External
        │  │     ├─ [calldata] [0x0]
        │  │     ├─ [storage address] 0x6ce4a1209f067bb71fbd98f37e1240f45beaf8fb22c39628cb2accfe4fb5bcb
        │  │     ├─ [caller address] 0x73668021e0dfc00aa573e654a0763dca79dbbfb5a60e4987f71a3472026994a
        │  │     ├─ [call type] Call
        │  │     └─ [call result] success: []
        │  └─ [selector] 0x387cbcd4e21cd976a8be7d24cd16b8e65e1940f9e06ded4f42798cf64c089be
        │     ├─ [entry point type] External
        │     ├─ [calldata] [0x0]
        │     ├─ [storage address] 0x6ce4a1209f067bb71fbd98f37e1240f45beaf8fb22c39628cb2accfe4fb5bcb
        │     ├─ [caller address] 0x29e61a26d304881d5944a9ab3446685cb791af1ff827f9d4bc34489a2ff9629
        │     ├─ [call type] Call
        │     └─ [call result] success: []
        └─ [selector] 0x32564d7e0fe091d49b4c20f4632191e4ed6986bf993849879abfef9465def25
           ├─ [entry point type] External
           ├─ [calldata] [0x5, 0x1, 0x2, 0x3, 0x4, 0x5]
           ├─ [storage address] 0x29e61a26d304881d5944a9ab3446685cb791af1ff827f9d4bc34489a2ff9629
           ├─ [caller address] 0x1724987234973219347210837402
           ├─ [call type] Call
           └─ [call result] panic: [0x1, 0x2, 0x3, 0x4, 0x5]
        "};

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
        {debugging_trace}

        [PASS] trace_info_integrationtest::test_trace::test_debugging_trace_success (gas: ~324)

        Success data:
            (0x1, 0x2, 0x3, 0x4, 0x5)

        note: run with `SNFORGE_BACKTRACE=1` environment variable to display a backtrace
        {debugging_trace}

        Running 0 test(s) from src/
        Tests: 1 passed, 1 failed, 0 skipped, 0 ignored, 0 filtered out

        Failures:
            trace_info_integrationtest::test_trace::test_debugging_trace_fail
        ", debugging_trace = DEBUGGING_TRACE},
    );
}
