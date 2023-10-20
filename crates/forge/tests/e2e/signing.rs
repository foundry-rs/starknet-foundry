use crate::assert_stdout_contains;
use crate::e2e::common::runner::{runner, setup_package};
use indoc::indoc;

#[test]
fn signing() {
    let temp = setup_package("signing");
    let snapbox = runner().arg("signing");

    let output = snapbox.current_dir(&temp).assert().code(0);

    assert_stdout_contains!(
        output,
        indoc! {r#"
        [..]Updating[..]
        [..]Compiling[..]
        [..]Finished[..]


        Collected 1 test(s) from signing package
        Running 1 test(s) from src/
        [PASS] signing::test
        Tests: 1 passed, 0 failed, 0 skipped
        "#}
    );
}
