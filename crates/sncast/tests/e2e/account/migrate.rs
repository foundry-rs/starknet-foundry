use crate::helpers::{fixtures::get_accounts_path, runner::runner};
use camino::Utf8Path;
use indoc::{formatdoc, indoc};
use std::fs;
use tempfile::tempdir;

#[tokio::test]
pub async fn test_happy_case() {
    let tempdir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts_v1_schema.json";
    let accounts_file_source_path =
        get_accounts_path(Utf8Path::new("tests/data/accounts").join(accounts_file));
    let accounts_file_tempdir_path = tempdir.path().join(accounts_file);
    fs::copy(accounts_file_source_path, &accounts_file_tempdir_path).unwrap();

    let args = vec!["--accounts-file", accounts_file, "account", "migrate"];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().stdout_eq(formatdoc!(
        r"
        Success: Accounts file migrated from version v1 to version v2
        V1 Backup: {accounts_file}.v1.bak
        "
    ));

    let contents =
        fs::read_to_string(accounts_file_tempdir_path).expect("Unable to read created file");
    insta::assert_snapshot!(contents);
}

#[tokio::test]
pub async fn test_happy_case_migration_not_required() {
    let tempdir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts_v2_schema.json";
    let accounts_file_source_path =
        get_accounts_path(Utf8Path::new("tests/data/accounts").join(accounts_file));
    let accounts_file_tempdir_path = tempdir.path().join(accounts_file);
    fs::copy(accounts_file_source_path, &accounts_file_tempdir_path).unwrap();

    let args = vec!["--accounts-file", accounts_file, "account", "migrate"];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().stderr_eq("").stdout_eq(indoc!(
        r"
        Success: Accounts file is already the latest version v2
        "
    ));
}

#[tokio::test]
pub async fn test_happy_case_empty_file() {
    let tempdir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "empty_accounts.json";
    let accounts_file_source_path =
        get_accounts_path(Utf8Path::new("tests/data/accounts").join(accounts_file));
    let accounts_file_tempdir_path = tempdir.path().join(accounts_file);
    fs::copy(accounts_file_source_path, &accounts_file_tempdir_path).unwrap();

    let args = vec!["--accounts-file", accounts_file, "account", "migrate"];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().stdout_eq(formatdoc!(
        r"
        Success: Accounts file migrated from version v1 to version v2
        V1 Backup: {accounts_file}.v1.bak
        "
    ));

    let contents =
        fs::read_to_string(accounts_file_tempdir_path).expect("Unable to read created file");
    insta::assert_snapshot!(contents);
}

#[tokio::test]
pub async fn test_invalid_format() {
    let tempdir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "invalid_format.json";
    let accounts_file_source_path =
        get_accounts_path(Utf8Path::new("tests/data/accounts").join(accounts_file));
    let accounts_file_tempdir_path = tempdir.path().join(accounts_file);
    fs::copy(accounts_file_source_path, &accounts_file_tempdir_path).unwrap();

    let args = vec!["--accounts-file", accounts_file, "account", "migrate"];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().stderr_eq(indoc!(
        r"
        Command: account migrate
        Error: invalid schema of field alpha-sepolia.? in the accounts file

        Caused by:
            expected `,` or `}` at line 8 column 9
        "
    ));
}

#[tokio::test]
pub async fn test_unsupported_version() {
    let tempdir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "unsupported_version.json";
    let accounts_file_source_path =
        get_accounts_path(Utf8Path::new("tests/data/accounts").join(accounts_file));
    let accounts_file_tempdir_path = tempdir.path().join(accounts_file);
    fs::copy(accounts_file_source_path, &accounts_file_tempdir_path).unwrap();

    let args = vec!["--accounts-file", accounts_file, "account", "migrate"];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().stderr_eq(indoc!(
        r"
        Command: account migrate
        Error: accounts file schema version 5 is not supported
        "
    ));
}
