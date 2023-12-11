use crate::helpers::fixtures::default_cli_args;
use crate::helpers::runner::runner;
use sncast::helpers::constants::DEFAULT_MULTICALL_CONTENTS;
use tempfile::tempdir;

#[tokio::test]
async fn test_happy_case_stdout() {
    let mut args = default_cli_args();

    args.append(&mut vec!["multicall", "new"]);

    let snapbox = runner(&args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");

    assert!(out.stderr.is_empty());
    assert!(stdout_str.contains(DEFAULT_MULTICALL_CONTENTS));
}

#[tokio::test]
async fn test_happy_case_file() {
    let mut args = default_cli_args();

    let tmp_dir = tempdir().expect("failed to create temporary directory");
    let tmp_path = tmp_dir.path().join("multicall.toml");
    let tmp_path = tmp_path.to_str().expect("failed to convert path to string");

    args.append(&mut vec!["multicall", "new", "--output-path", tmp_path]);

    let snapbox = runner(&args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let contents =
        std::fs::read_to_string(tmp_path).expect("Should have been able to read the file");
    assert!(out.stderr.is_empty());

    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");

    assert!(stdout_str.contains("path: "));
    assert!(stdout_str.contains("content: "));
    assert!(contents.contains(DEFAULT_MULTICALL_CONTENTS));
}

#[tokio::test]
async fn test_directory_non_existent() {
    let mut args = default_cli_args();

    let tmp_dir = tempdir().expect("failed to create temporary directory");
    let tmp_path = tmp_dir
        .path()
        .join("non_existent_directory")
        .join("multicall.toml");
    let tmp_path = tmp_path.to_str().expect("failed to convert path to string");

    args.append(&mut vec!["multicall", "new", "--output-path", tmp_path]);

    let snapbox = runner(&args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");
    let stderr_str =
        std::str::from_utf8(&out.stderr).expect("failed to convert command output to string");
    assert!(stdout_str.is_empty());
    assert!(stderr_str.contains("No such file or directory"));
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

    let snapbox = runner(&args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");
    let stderr_str =
        std::str::from_utf8(&out.stderr).expect("failed to convert command output to string");
    assert!(stdout_str.is_empty());
    assert!(stderr_str.contains("Output file cannot be a directory"));
}
