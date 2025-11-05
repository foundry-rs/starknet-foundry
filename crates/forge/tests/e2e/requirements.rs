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

#[test]
#[cfg_attr(not(feature = "scarb_2_11_0"), ignore)]
fn test_warning_on_scarb_version_below_recommended() {
    let temp = setup_package("simple_package");
    let output = runner(&temp).arg("check-requirements").assert();

    assert_stdout_contains(
        output,
        indoc! {r"
    Checking requirements

    ⚠️  Scarb Version 2.11.0 doesn't satisfy minimal recommended [..]
    ✅ Universal Sierra Compiler [..]
    "},
    );
}
