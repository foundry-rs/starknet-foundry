use crate::e2e::common::runner::{setup_package, test_runner};
use indoc::indoc;
use shared::test_utils::output_assert::assert_stderr_contains;

#[test]
fn test_does_not_work_with_exact_flag() {
    let temp = setup_package("simple_package");
    let output = test_runner(&temp)
        .args(["--partition", "3/3", "--workspace", "--exact"])
        .assert()
        .code(2);

    assert_stderr_contains(
        output,
        indoc! {r"
        error: the argument '--partition <PARTITION>' cannot be used with '--exact'
    "},
    );
}
