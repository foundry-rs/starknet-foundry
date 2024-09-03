use super::common::runner::{setup_package, test_runner};
use forge_runner::coverage_api::{COVERAGE_DIR, OUTPUT_FILE_NAME};

#[test]
#[ignore] // TODO(#2426) this will only work for scarb 2.8.0 or above
fn test_coverage_project() {
    let temp = setup_package("coverage_project");

    test_runner(&temp).arg("--coverage").assert().success();

    assert!(temp.join(COVERAGE_DIR).join(OUTPUT_FILE_NAME).is_file());

    // Check if it doesn't crash in case some data already exists
    test_runner(&temp).arg("--coverage").assert().success();
}
