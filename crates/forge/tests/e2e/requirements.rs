use crate::e2e::common::runner::{runner, setup_package};
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;

#[test]
fn happy_path() {
    let temp = setup_package("simple_package");
    let output = runner(&temp).arg("check-requirements").assert();

    assert_stdout_contains(
        output,
        indoc! {r"
    Checking requirements

    ✅ Scarb [..]
    ✅ Universal Sierra Compiler [..]

    "},
    );
}

#[cfg(feature = "scarb_2_13_1")]
#[test]
fn test_warning_on_scarb_version_below_recommended() {
    let temp = setup_package("simple_package");
    let output = runner(&temp).arg("check-requirements").assert();

    assert_stdout_contains(
        output,
        indoc! {r"
    Checking requirements

    ⚠️  Scarb Version 2.13.1 doesn't satisfy minimal recommended [..]
    ✅ Universal Sierra Compiler [..]
    "},
    );
}
