use crate::e2e::common::runner::{runner, setup_package};
use indoc::{formatdoc, indoc};
use scarb_api::version::scarb_version;
use semver::Version;
use shared::test_utils::output_assert::assert_stdout_contains;

#[test]
fn happy_path() {
    let temp = setup_package("simple_package");
    let output = runner(&temp).arg("check-requirements").assert();

    let scarb_version = scarb_version().unwrap().scarb;

    let rust_check = if scarb_version < Version::new(2, 10, 0) {
        indoc! {"
        Checking requirements

        ✅ Rust [..]
        "}
    } else {
        ""
    };

    assert_stdout_contains(
        output,
        formatdoc! {r"
    Checking requirements

    ✅ Scarb [..]
    ✅ Universal Sierra Compiler [..]
    {}
    ", rust_check},
    );
}

#[test]
#[cfg_attr(not(feature = "scarb_2_10_0"), ignore)]
fn test_warning_on_scarb_version_below_recommended() {
    let temp = setup_package("simple_package");
    let output = runner(&temp).arg("check-requirements").assert();

    assert_stdout_contains(
        output,
        indoc! {r"
    Checking requirements

    ⚠️  Scarb Version 2.10.0 doesn't satisfy minimal recommended [..]
    ✅ Universal Sierra Compiler [..]
    "},
    );
}
