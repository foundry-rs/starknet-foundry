use crate::e2e::common::runner::{setup_package, test_runner};
use indoc::indoc;
use scarb_api::ScarbCommand;
use shared::test_utils::output_assert::assert_stdout_contains;
use snapbox::cmd::Command as SnapboxCommand;

#[test]
fn simple() {
    let temp = setup_package("declare_from_file");

    SnapboxCommand::from_std(
        ScarbCommand::new()
            .current_dir(temp.path())
            .arg("build")
            .command(),
    )
    .assert()
    .success();

    let output = test_runner(&temp).arg("simple").assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]

        Collected 1 test(s) from declare_from_file package
        Running 0 test(s) from src/
        Running 1 test(s) from tests/
        [PASS] declare_from_file_integrationtest::tests::simple [..]
        Tests: 1 passed, 0 failed, 0 ignored, 1 filtered out
        "},
    );
}

#[test]
fn already_declared() {
    let temp = setup_package("declare_from_file");

    SnapboxCommand::from_std(
        ScarbCommand::new()
            .current_dir(temp.path())
            .arg("build")
            .command(),
    )
    .assert()
    .success();

    let output = test_runner(&temp)
        .arg("already_declared")
        .assert()
        .success();

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]

        Collected 1 test(s) from declare_from_file package
        Running 0 test(s) from src/
        Running 1 test(s) from tests/
        [PASS] declare_from_file_integrationtest::tests::already_declared [..]
        Tests: 1 passed, 0 failed, 0 ignored, 1 filtered out
        "},
    );
}

#[test]
fn missing_file() {
    let temp = setup_package("declare_from_file_failures");

    let output = test_runner(&temp).arg("missing_file").assert().code(1);

    assert_stdout_contains(
        output,
        indoc! {r#"
        [FAIL] declare_from_file_failures_integrationtest::tests::missing_file

        Failure data:
            "Failed to read Sierra file at data/missing.contract_class.json: [..]"
        "#},
    );
}

#[test]
fn invalid_json() {
    let temp = setup_package("declare_from_file_failures");

    let output = test_runner(&temp).arg("invalid_json").assert().code(1);

    assert_stdout_contains(
        output,
        indoc! {r#"
        [FAIL] declare_from_file_failures_integrationtest::tests::invalid_json

        Failure data:
            "Failed to parse Sierra contract class JSON at data/invalid_contract_class.json: [..]"
        "#},
    );
}
