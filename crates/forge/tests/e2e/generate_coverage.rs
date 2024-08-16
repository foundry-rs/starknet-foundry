use super::common::runner::{setup_package, test_runner};
use forge_runner::coverage_api::{COVERAGE_DIR, OUTPUT_FILE_NAME};

#[test]
#[ignore]
fn simple_package_generate_coverage() {
    let temp = setup_package("simple_package");

    test_runner(&temp)
        .arg("--generate-coverage")
        .assert()
        .code(1);

    assert!(temp.join(COVERAGE_DIR).join(OUTPUT_FILE_NAME).is_file());

    // Check if it doesn't crash in case some data already exists
    test_runner(&temp)
        .arg("--generate-coverage")
        .assert()
        .code(1);
}
