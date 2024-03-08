use super::common::runner::{setup_package, test_runner};
use forge_runner::profiler_api::PROFILE_DIR;

#[test]
fn simple_package_build_profile() {
    let temp = setup_package("simple_package");

    test_runner(&temp).arg("--build-profile").assert().code(1);

    assert!(temp
        .join(PROFILE_DIR)
        .join("simple_package::tests::test_fib.pb.gz")
        .exists());
    assert!(!temp
        .join(PROFILE_DIR)
        .join("tests::test_simple::test_failing.pb.gz")
        .exists());
    assert!(!temp
        .join(PROFILE_DIR)
        .join("simple_package::tests::ignored_test.pb.gz")
        .exists());
    assert!(temp
        .join(PROFILE_DIR)
        .join("tests::ext_function_test::test_simple.pb.gz")
        .exists());

    // Check if it doesn't crash in case some data already exists
    test_runner(&temp).arg("--build-profile").assert().code(1);
}
