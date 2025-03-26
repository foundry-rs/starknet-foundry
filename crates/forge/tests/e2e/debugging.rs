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
        │  ├─ [calldata] [0x2, 0x417934062c9a072bc24cd5ccd4c18a58e9781f9ed06f78205d63c222efa8de8, 0x2, 0x615fa271ad6fab6b379f68f64787160846fbb4b8e639722232d4a3e9812675e, 0x0, 0x615fa271ad6fab6b379f68f64787160846fbb4b8e639722232d4a3e9812675e, 0x0, 0x615fa271ad6fab6b379f68f64787160846fbb4b8e639722232d4a3e9812675e, 0x0]
        │  ├─ [storage address] 0x26cbd91a1f45da60f4364c248bf52a5babf31c27dab750378e7e5b0d540c98e
        │  ├─ [caller address] 0x1724987234973219347210837402
        │  ├─ [call type] Call
        │  ├─ [call result] success: []
        │  ├─ [selector] execute_calls
        │  │  ├─ [contract name] SimpleContract
        │  │  ├─ [entry point type] External
        │  │  ├─ [calldata] [0x2, 0x615fa271ad6fab6b379f68f64787160846fbb4b8e639722232d4a3e9812675e, 0x0, 0x615fa271ad6fab6b379f68f64787160846fbb4b8e639722232d4a3e9812675e, 0x0]
        │  │  ├─ [storage address] 0x417934062c9a072bc24cd5ccd4c18a58e9781f9ed06f78205d63c222efa8de8
        │  │  ├─ [caller address] 0x26cbd91a1f45da60f4364c248bf52a5babf31c27dab750378e7e5b0d540c98e
        │  │  ├─ [call type] Call
        │  │  ├─ [call result] success: []
        │  │  ├─ [selector] execute_calls
        │  │  │  ├─ [contract name] SimpleContract
        │  │  │  ├─ [entry point type] External
        │  │  │  ├─ [calldata] [0x0]
        │  │  │  ├─ [storage address] 0x615fa271ad6fab6b379f68f64787160846fbb4b8e639722232d4a3e9812675e
        │  │  │  ├─ [caller address] 0x417934062c9a072bc24cd5ccd4c18a58e9781f9ed06f78205d63c222efa8de8
        │  │  │  ├─ [call type] Call
        │  │  │  └─ [call result] success: []
        │  │  └─ [selector] execute_calls
        │  │     ├─ [contract name] SimpleContract
        │  │     ├─ [entry point type] External
        │  │     ├─ [calldata] [0x0]
        │  │     ├─ [storage address] 0x615fa271ad6fab6b379f68f64787160846fbb4b8e639722232d4a3e9812675e
        │  │     ├─ [caller address] 0x417934062c9a072bc24cd5ccd4c18a58e9781f9ed06f78205d63c222efa8de8
        │  │     ├─ [call type] Call
        │  │     └─ [call result] success: []
        │  └─ [selector] execute_calls
        │     ├─ [contract name] SimpleContract
        │     ├─ [entry point type] External
        │     ├─ [calldata] [0x0]
        │     ├─ [storage address] 0x615fa271ad6fab6b379f68f64787160846fbb4b8e639722232d4a3e9812675e
        │     ├─ [caller address] 0x26cbd91a1f45da60f4364c248bf52a5babf31c27dab750378e7e5b0d540c98e
        │     ├─ [call type] Call
        │     └─ [call result] success: []
        └─ [selector] fail
           ├─ [contract name] SimpleContract
           ├─ [entry point type] External
           ├─ [calldata] [0x5, 0x1, 0x2, 0x3, 0x4, 0x5]
           ├─ [storage address] 0x26cbd91a1f45da60f4364c248bf52a5babf31c27dab750378e7e5b0d540c98e
           ├─ [caller address] 0x1724987234973219347210837402
           ├─ [call type] Call
           └─ [call result] panic: [0x1, 0x2, 0x3, 0x4, 0x5]
        "};

#[test]
fn debugging_trace() {
    let temp = setup_package("debugging");

    let output = test_runner(&temp).assert().code(1);

    let stdout = std::str::from_utf8(&output.get_output().stdout).unwrap();

    println!("XXX");
    println!("{}", stdout);
    println!("XXX");

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
