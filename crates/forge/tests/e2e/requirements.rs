use crate::e2e::common::runner::{runner, setup_package};
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;

#[test]
fn happy_path() {
    let temp = setup_package("simple_package");
    let output = runner(&temp).arg("validate-requirements").assert();

    assert_stdout_contains(
        output,
        indoc! {r"
    Validating requirements

    ✅ Rust [..]
    ✅ Scarb [..]
    ✅ Universal Sierra Compiler [..]
    "},
    );
}
