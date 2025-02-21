use super::common::runner::{setup_package, test_runner};
use assert_fs::fixture::{FileWriteStr, PathChild};
use forge_runner::coverage_api::{COVERAGE_DIR, OUTPUT_FILE_NAME};
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;
use std::fs;
use toml_edit::{DocumentMut, value};

#[test]
#[cfg_attr(not(feature = "scarb_2_8_3"), ignore)]
fn test_coverage_project() {
    let temp = setup_package("coverage_project");

    test_runner(&temp).arg("--coverage").assert().success();

    assert!(temp.join(COVERAGE_DIR).join(OUTPUT_FILE_NAME).is_file());

    // Check if it doesn't crash in case some data already exists
    test_runner(&temp).arg("--coverage").assert().success();
}

#[test]
#[cfg_attr(not(feature = "scarb_2_8_3"), ignore)]
fn test_coverage_project_and_pass_args() {
    let temp = setup_package("coverage_project");

    test_runner(&temp)
        .arg("--coverage")
        .arg("--")
        .arg("--output-path")
        .arg("./my_file.lcov")
        .assert()
        .success();

    assert!(temp.join("my_file.lcov").is_file());
}

#[test]
#[cfg_attr(not(feature = "scarb_2_7_1"), ignore)]
fn test_fail_on_scarb_version_lt_2_8_0() {
    let temp = setup_package("coverage_project");

    let output = test_runner(&temp).arg("--coverage").assert().failure();

    assert_stdout_contains(
        output,
        "[ERROR] Coverage generation requires scarb version >= 2.8.0\n",
    );
}

#[test]
#[cfg_attr(not(feature = "scarb_2_8_3"), ignore)]
fn test_fail_wrong_set_up() {
    let temp = setup_package("coverage_project");

    let manifest_path = temp.child("Scarb.toml");

    let mut scarb_toml = fs::read_to_string(&manifest_path)
        .unwrap()
        .parse::<DocumentMut>()
        .unwrap();

    scarb_toml["profile"]["dev"]["cairo"]["unstable-add-statements-code-locations-debug-info"] =
        value(false);

    manifest_path.write_str(&scarb_toml.to_string()).unwrap();

    let output = test_runner(&temp).arg("--coverage").assert().failure();

    assert_stdout_contains(
        output,
        indoc! {
            "[ERROR] Scarb.toml must have the following Cairo compiler configuration to run coverage:

            [profile.dev.cairo]
            unstable-add-statements-functions-debug-info = true
            unstable-add-statements-code-locations-debug-info = true
            inlining-strategy = \"avoid\"
            ... other entries ...

            "
        },
    );
}
