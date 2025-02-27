use super::common::runner::{setup_package, test_runner};
use forge_runner::profiler_api::PROFILE_DIR;

#[test]
#[cfg(not(target_os = "windows"))]
fn simple_package_build_profile() {
    let temp = setup_package("simple_package");

    test_runner(&temp).arg("--build-profile").assert().code(1);

    assert!(
        temp.join(PROFILE_DIR)
            .join("simple_package_tests_test_fib.pb.gz")
            .is_file()
    );
    assert!(
        !temp
            .join(PROFILE_DIR)
            .join("simple_package_integrationtest_test_simple_test_failing.pb.gz")
            .is_file()
    );
    assert!(
        !temp
            .join(PROFILE_DIR)
            .join("simple_package_tests_ignored_test.pb.gz")
            .is_file()
    );
    assert!(
        temp.join(PROFILE_DIR)
            .join("simple_package_integrationtest_ext_function_test_test_simple.pb.gz")
            .is_file()
    );

    // Check if it doesn't crash in case some data already exists
    test_runner(&temp).arg("--build-profile").assert().code(1);
}

#[test]
#[cfg(not(target_os = "windows"))]
fn simple_package_build_profile_and_pass_args() {
    let temp = setup_package("simple_package");

    test_runner(&temp)
        .arg("--build-profile")
        .arg("--")
        .arg("--output-path")
        .arg("my_file.pb.gz")
        .assert()
        .code(1);

    assert!(temp.join("my_file.pb.gz").is_file());
}
