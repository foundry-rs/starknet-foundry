use crate::helpers::constants::ACCOUNT_FILE_PATH;
use crate::helpers::runner::runner;
use indoc::{formatdoc, indoc};
use shared::test_utils::output_assert::{AsOutput, assert_stderr_contains, assert_stdout_contains};
use sncast::helpers::constants::DEFAULT_MULTICALL_CONTENTS;
use tempfile::tempdir;

#[tokio::test]
async fn test_happy_case_file() {
    let tmp_dir = tempdir().expect("Failed to create temporary directory");
    let multicall_toml_file = "multicall.toml";

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "multicall",
        "new",
        multicall_toml_file,
    ];

    let snapbox = runner(&args).current_dir(tmp_dir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        Success: Multicall template created successfully

        Path:    multicall.toml
        Content: [..]
        "},
    );

    let contents = std::fs::read_to_string(tmp_dir.path().join(multicall_toml_file))
        .expect("Should have been able to read the file");

    assert!(contents.contains(DEFAULT_MULTICALL_CONTENTS));
}

#[tokio::test]
async fn test_no_output_path_specified() {
    let args = vec!["--accounts-file", ACCOUNT_FILE_PATH, "multicall", "new"];

    let snapbox = runner(&args);
    let output = snapbox.assert().failure();

    let expected = indoc! {r"
    error: the following required arguments were not provided:
      <OUTPUT_PATH>

    Usage: sncast[..] multicall new <OUTPUT_PATH>

    For more information, try '--help'.
    "};

    assert!(output.as_stdout().is_empty());
    assert_stderr_contains(output, expected);
}

#[tokio::test]
async fn test_directory_non_existent() {
    let tmp_dir = tempdir().expect("failed to create temporary directory");
    let multicall_toml_path = "non_existent_directory/multicall.toml";

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "multicall",
        "new",
        multicall_toml_path,
    ];

    let snapbox = runner(&args).current_dir(tmp_dir.path());
    let output = snapbox.assert().success();

    assert!(output.as_stdout().is_empty());

    let expected_file_error = "No such file or directory [..]";

    assert_stderr_contains(
        output,
        formatdoc! {r"
        Command: multicall new
        Error: {}
        ", expected_file_error},
    );
}

#[tokio::test]
async fn test_file_invalid_path() {
    let tmp_dir = tempdir().expect("failed to create temporary directory");
    let tmp_path = tmp_dir
        .path()
        .to_str()
        .expect("failed to convert path to string");

    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "multicall",
        "new",
        tmp_path,
    ];

    let snapbox = runner(&args).current_dir(tmp_dir.path());
    let output = snapbox.assert().success();

    assert!(output.as_stdout().is_empty());
    assert_stderr_contains(
        output,
        indoc! {r"
        Command: multicall new
        Error: Output file cannot be a directory[..]
        "},
    );
}
