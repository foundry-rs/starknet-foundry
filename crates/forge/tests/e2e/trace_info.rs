use crate::assert_stdout_contains;
use crate::e2e::common::runner::{setup_package, test_runner};
use indoc::indoc;

#[test]
fn trace_info_print() {
    let temp = setup_package("trace_info");
    let snapbox = test_runner();

    let output = snapbox.current_dir(&temp).assert().success();

    assert_stdout_contains!(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 1 test(s) from trace_info package
        Running 0 test(s) from src/
        Running 1 test(s) from tests/
        original value: [92290246192609328148519485807009558629], converted to a string: [Entry Point Type]
        original value: [5005878964882661740], converted to a string: [External]
        original value: [396383589137045581302423422359764530380999520114], converted to a string: [Entry Point Selector]
        original value: [1024936690255032842875463015856695803908824911639649964560344728797058303195]
        original value: [4855281086078481505], converted to a string: [Calldata]
        original value: [10]
        original value: [433322228497498563607775107824841587], converted to a string: [Storage Address]
        original value: [2656140848091984552801736834593776881381027675231878699139041248628012329534]
        original value: [1366640130632659050641915078079347], converted to a string: [Caller Address]
        original value: [469394814521890341860918960550914]
        original value: [1242951957743815716965], converted to a string: [Call Type]
        original value: [1130458220], converted to a string: [Call]
        [PASS] tests::available_gas::test_trace_info_print, gas: ~1848
        Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
        "}
    );
}
