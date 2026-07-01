use crate::e2e::common::runner::{setup_package, test_runner};
use indoc::indoc;
use scarb_api::ScarbCommand;
use shared::test_utils::output_assert::assert_stdout_contains;
use snapbox::cmd::Command as SnapboxCommand;

#[test]
fn declare_from_file() {
    let temp = setup_package("declare_from_file");

    SnapboxCommand::from_std(
        ScarbCommand::new()
            .current_dir(temp.path())
            .arg("build")
            .command(),
    )
    .assert()
    .success();

    let output = test_runner(&temp).assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 2 test(s) from declare_from_file package
        Running 0 test(s) from src/
        Running 2 test(s) from tests/
        [PASS] declare_from_file_integrationtest::tests::simple [..]
        [PASS] declare_from_file_integrationtest::tests::already_declared [..]
        Tests: 2 passed, 0 failed, 0 ignored, 0 filtered out
        "},
    );
}
