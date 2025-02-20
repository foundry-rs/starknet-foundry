use crate::e2e::common::runner::{runner, setup_package};
use indoc::formatdoc;
use scarb_api::ScarbCommand;
use semver::Version;
use shared::test_utils::output_assert::assert_stdout_contains;

#[test]
fn happy_path() {
    let temp = setup_package("simple_package");
    let output = runner(&temp).arg("check-requirements").assert();

    let scarb_version = ScarbCommand::version().run().unwrap().scarb;

    let rust_check = if scarb_version < Version::new(2, 10, 0) {
        "✅ Rust [..]"
    } else {
        ""
    };

    assert_stdout_contains(
        output,
        formatdoc! {r"
    Checking requirements
    {}
    ✅ Scarb [..]
    ✅ Universal Sierra Compiler [..]
    ", rust_check},
    );
}
