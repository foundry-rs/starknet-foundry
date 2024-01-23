use forge_runner::trace_data::TRACE_DIR;

use crate::e2e::common::runner::{setup_package, test_runner};

#[test]
fn simple_package_save_trace() {
    let temp = setup_package("simple_package");
    let snapbox = test_runner();
    snapbox
        .current_dir(&temp)
        .arg("--save-trace-data")
        .assert()
        .code(1);

    assert!(temp
        .join(TRACE_DIR)
        .join("simple_package::tests::test_fib")
        .exists());
    assert!(!temp
        .join(TRACE_DIR)
        .join("tests::test_simple::test_failing")
        .exists());
    assert!(!temp
        .join(TRACE_DIR)
        .join("simple_package::tests::ignored_test")
        .exists());
    assert!(temp
        .join(TRACE_DIR)
        .join("tests::ext_function_test::test_simple")
        .exists());

    // Check if it doesn't crash in case some data already exists
    let snapbox = test_runner();
    snapbox
        .current_dir(&temp)
        .arg("--save-trace-data")
        .assert()
        .code(1);
}
