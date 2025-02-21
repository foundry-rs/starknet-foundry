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

    ✅ Rust [..]
    ✅ Scarb [..]
    ✅ Universal Sierra Compiler [..]
    "},
    );
}

#[test]
#[cfg_attr(not(feature = "scarb_2_7_1"), ignore)]
fn test_warning_on_outdated_scarb() {
    let temp = setup_package("simple_package");
    let output = runner(&temp).arg("check-requirements").assert();

    assert_stdout_contains(
        output,
        indoc! {r"
    Checking requirements

    ✅ Rust [..]
    ✅ Scarb [..]
    ✅ Universal Sierra Compiler [..]
    ⚠️  Scarb is outdated. We recommend updating to at least version [..]
    "},
    );
}
