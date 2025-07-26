use super::common::runner::{setup_package, test_runner};
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;

#[test]
fn trace_info_print() {
    let temp = setup_package("trace");

    let output = test_runner(&temp).assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]

        Collected 1 test(s) from trace_info package
        Running 0 test(s) from src/
        Running 1 test(s) from tests/
        Entry point type: External
        Selector: [..]
        Calldata: []
        Storage address: [..]
        Caller address: 0
        Call type: Call
        Nested Calls: [
            (
                Entry point type: External
                Selector: [..]
                Calldata: [..]
                Storage address: [..]
                Caller address: [..]
                Call type: Call
                Nested Calls: [
                    (
                        Entry point type: External
                        Selector: [..]
                        Calldata: [..]
                        Storage address: [..]
                        Caller address: [..]
                        Call type: Call
                        Nested Calls: [
                            (
                                Entry point type: External
                                Selector: [..]
                                Calldata: [0]
                                Storage address: [..]
                                Caller address: [..]
                                Call type: Call
                                Nested Calls: []
                                Call Result: Success: []
                            ),
                            (
                                Entry point type: External
                                Selector: [..]
                                Calldata: [0]
                                Storage address: [..]
                                Caller address: [..]
                                Call type: Call
                                Nested Calls: []
                                Call Result: Success: []
                            )
                        ]
                        Call Result: Success: []
                    ),
                    (
                        Entry point type: External
                        Selector: [..]
                        Calldata: [0]
                        Storage address: [..]
                        Caller address: [..]
                        Call type: Call
                        Nested Calls: []
                        Call Result: Success: []
                    )
                ]
                Call Result: Success: []
            ),
            (
                Entry point type: External
                Selector: 1423007881864269398513176851135908567621420218646181695002463829511917924133
                Calldata: [5, 1, 2, 3, 4, 5]
                Storage address: [..]
                Caller address: 469394814521890341860918960550914
                Call type: Call
                Nested Calls: []
                Call Result: Failure: [1, 2, 3, 4, 5]
            )
        ]
        Call Result: Success: []
        
        [PASS] trace_info_integrationtest::test_trace::test_trace
        Tests: 1 passed, 0 failed, 0 ignored, 0 filtered out
        "},
    );
}
