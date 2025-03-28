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
        ├─ [selector] execute_calls
        │  ├─ [contract name] SimpleContract
        │  ├─ [entry point type] External
        │  ├─ [calldata] [0x2, 0x634cf632813aca745d024ee244aab954461a7341b610b103fa7569bf1e14a5e, 0x2, 0x38767c97f072a291507aa962d6d92b04ae7b4e01c406717f5485b7a86fbdde7, 0x0, 0x38767c97f072a291507aa962d6d92b04ae7b4e01c406717f5485b7a86fbdde7, 0x0, 0x38767c97f072a291507aa962d6d92b04ae7b4e01c406717f5485b7a86fbdde7, 0x0]
        │  ├─ [storage address] 0x5ab84515ea91a99261961ba72888d663606612958a363e8538302a27398bc29
        │  ├─ [caller address] 0x1724987234973219347210837402
        │  ├─ [call type] Call
        │  ├─ [call result] success: []
        │  ├─ [selector] execute_calls
        │  │  ├─ [contract name] SimpleContract
        │  │  ├─ [entry point type] External
        │  │  ├─ [calldata] [0x2, 0x38767c97f072a291507aa962d6d92b04ae7b4e01c406717f5485b7a86fbdde7, 0x0, 0x38767c97f072a291507aa962d6d92b04ae7b4e01c406717f5485b7a86fbdde7, 0x0]
        │  │  ├─ [storage address] 0x634cf632813aca745d024ee244aab954461a7341b610b103fa7569bf1e14a5e
        │  │  ├─ [caller address] 0x5ab84515ea91a99261961ba72888d663606612958a363e8538302a27398bc29
        │  │  ├─ [call type] Call
        │  │  ├─ [call result] success: []
        │  │  ├─ [selector] execute_calls
        │  │  │  ├─ [contract name] SimpleContract
        │  │  │  ├─ [entry point type] External
        │  │  │  ├─ [calldata] [0x0]
        │  │  │  ├─ [storage address] 0x38767c97f072a291507aa962d6d92b04ae7b4e01c406717f5485b7a86fbdde7
        │  │  │  ├─ [caller address] 0x634cf632813aca745d024ee244aab954461a7341b610b103fa7569bf1e14a5e
        │  │  │  ├─ [call type] Call
        │  │  │  └─ [call result] success: []
        │  │  └─ [selector] execute_calls
        │  │     ├─ [contract name] SimpleContract
        │  │     ├─ [entry point type] External
        │  │     ├─ [calldata] [0x0]
        │  │     ├─ [storage address] 0x38767c97f072a291507aa962d6d92b04ae7b4e01c406717f5485b7a86fbdde7
        │  │     ├─ [caller address] 0x634cf632813aca745d024ee244aab954461a7341b610b103fa7569bf1e14a5e
        │  │     ├─ [call type] Call
        │  │     └─ [call result] success: []
        │  └─ [selector] execute_calls
        │     ├─ [contract name] SimpleContract
        │     ├─ [entry point type] External
        │     ├─ [calldata] [0x0]
        │     ├─ [storage address] 0x38767c97f072a291507aa962d6d92b04ae7b4e01c406717f5485b7a86fbdde7
        │     ├─ [caller address] 0x5ab84515ea91a99261961ba72888d663606612958a363e8538302a27398bc29
        │     ├─ [call type] Call
        │     └─ [call result] success: []
        └─ [selector] fail
           ├─ [contract name] SimpleContract
           ├─ [entry point type] External
           ├─ [calldata] [0x5, 0x1, 0x2, 0x3, 0x4, 0x5]
           ├─ [storage address] 0x5ab84515ea91a99261961ba72888d663606612958a363e8538302a27398bc29
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

        [PASS] trace_info_integrationtest::test_trace::test_debugging_trace_success (l1_gas: ~0, l1_data_gas: ~288, l2_gas: ~1440000)

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
