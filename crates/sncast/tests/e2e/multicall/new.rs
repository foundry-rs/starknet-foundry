use crate::helpers::fixtures::default_cli_args;
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains, AsOutput};
use sncast::helpers::constants::DEFAULT_MULTICALL_CONTENTS;
use tempfile::tempdir;

#[tokio::test]
async fn test_happy_case_stdout() {
    let mut args = default_cli_args();

    args.append(&mut vec!["multicall", "new"]);

    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert!(output.as_stderr().is_empty());
    assert_stdout_contains(output, DEFAULT_MULTICALL_CONTENTS);
}

#[tokio::test]
async fn test_happy_case_file() {
    let mut args = default_cli_args();
    let tmp_dir = tempdir().expect("failed to create temporary directory");
    let multicall_toml_file = "multicall.toml";

    args.append(&mut vec![
        "multicall",
        "new",
        "--output-path",
        multicall_toml_file,
    ]);

    let snapbox = runner(&args).current_dir(tmp_dir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        command: multicall new
        content:[..]
        path: multicall.toml
        "},
    );
    
    let contents = std::fs::read_to_string(tmp_dir.path().join(multicall_toml_file))
        .expect("Should have been able to read the file");
    assert!(contents.contains(DEFAULT_MULTICALL_CONTENTS));
}

#[tokio::test]
async fn test_directory_non_existent() {
    let mut args = default_cli_args();

    let tmp_dir = tempdir().expect("failed to create temporary directory");
    let multicall_toml_path = "non_existent_directory/multicall.toml";

    args.append(&mut vec![
        "multicall",
        "new",
        "--output-path",
        multicall_toml_path,
    ]);

    let snapbox = runner(&args).current_dir(tmp_dir.path());
    let output = snapbox.assert().success();

    assert!(output.as_stdout().is_empty());
    assert_stderr_contains(
        output,
        indoc! {r"
        command: multicall new
        error: No such file or directory[..]
        "},
    );
}

#[tokio::test]
async fn test_file_invalid_path() {
    let mut args = default_cli_args();

    let tmp_dir = tempdir().expect("failed to create temporary directory");
    let tmp_path = tmp_dir
        .path()
        .to_str()
        .expect("failed to convert path to string");

    args.append(&mut vec!["multicall", "new", "--output-path", tmp_path]);

    let snapbox = runner(&args).current_dir(tmp_dir.path());
    let output = snapbox.assert().success();

    assert!(output.as_stdout().is_empty());
    assert_stderr_contains(
        output,
        indoc! {r"
        command: multicall new
        error: Output file cannot be a directory[..]
        "},
    );
}
