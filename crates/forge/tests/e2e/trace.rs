use crate::assert_stdout_contains;
use crate::e2e::common::runner::{setup_package, test_runner};
use indoc::indoc;

#[test]
fn trace_info_print() {
    let temp = setup_package("trace");
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
        
        Entry point type: External
        Selector: 1024936690255032842875463015856695803908824911639649964560344728797058303195
        Calldata: [10]
        Storage address: 2455907345917140204161489130051965975039431451193458942118883935209731300862
        Caller address: 469394814521890341860918960550914
        Call type: Call
        
        [PASS] tests::test_trace::test_trace_print (gas: ~[..])
        Tests: 1 passed, 0 failed, 0 skipped, 0 ignored, 0 filtered out
        "}
    );
}
