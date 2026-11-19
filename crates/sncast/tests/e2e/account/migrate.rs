use crate::helpers::{fixtures::get_accounts_path, runner::runner};
use camino::Utf8Path;
use indoc::{formatdoc, indoc};
use serde_json::{Value, json};
use std::fs;
use tempfile::tempdir;
use test_case::test_case;

#[tokio::test]
pub async fn test_happy_case() {
    let tempdir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts_v1_schema.json";
    let accounts_file_source_path =
        get_accounts_path(Utf8Path::new("tests/data/accounts").join(accounts_file));
    let accounts_file_tempdir_path = tempdir.path().join(accounts_file);
    fs::copy(accounts_file_source_path, &accounts_file_tempdir_path).unwrap();
    let original = fs::read(&accounts_file_tempdir_path).unwrap();

    let args = vec!["--accounts-file", accounts_file, "account", "migrate"];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(formatdoc!(
        r"
        Success: Accounts file migrated from version v1 to version v2
        V1 Backup: {accounts_file}.v1.bak
        "
    ));

    let contents =
        fs::read_to_string(accounts_file_tempdir_path).expect("Unable to read created file");
    insta::assert_snapshot!(contents);
    assert_eq!(
        fs::read(tempdir.path().join(format!("{accounts_file}.v1.bak"))).unwrap(),
        original
    );
}

#[tokio::test]
pub async fn test_happy_case_migration_not_required() {
    let tempdir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts_v2_schema.json";
    let accounts_file_source_path =
        get_accounts_path(Utf8Path::new("tests/data/accounts").join(accounts_file));
    let accounts_file_tempdir_path = tempdir.path().join(accounts_file);
    fs::copy(accounts_file_source_path, &accounts_file_tempdir_path).unwrap();
    let original = format!(
        "\n {} \n",
        fs::read_to_string(&accounts_file_tempdir_path).unwrap()
    );
    fs::write(&accounts_file_tempdir_path, &original).unwrap();

    let args = vec!["--accounts-file", accounts_file, "account", "migrate"];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stderr_eq("").stdout_eq(indoc!(
        r"
        Success: Accounts file uses the latest schema version v2
        "
    ));
    assert_eq!(
        fs::read_to_string(accounts_file_tempdir_path).unwrap(),
        original
    );
    assert!(
        !tempdir
            .path()
            .join(format!("{accounts_file}.v1.bak"))
            .exists()
    );
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

    snapbox.assert().success().stdout_eq(formatdoc!(
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
    let original = fs::read(&accounts_file_tempdir_path).unwrap();

    let args = vec!["--accounts-file", accounts_file, "account", "migrate"];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().failure().stderr_eq(indoc!(
        r"
        Command: account migrate
        Error: invalid accounts file `invalid_format.json`

        Caused by:
            not valid JSON: expected `,` or `}` at line 8 column 9
        "
    ));
    assert_eq!(fs::read(accounts_file_tempdir_path).unwrap(), original);
    assert!(
        !tempdir
            .path()
            .join(format!("{accounts_file}.v1.bak"))
            .exists()
    );
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

    snapbox.assert().failure().stderr_eq(indoc!(
        r"
        Command: account migrate
        Error: invalid accounts file `unsupported_version.json`

        Caused by:
            schema version 5 is not supported
        "
    ));
}

#[test_case("{}\n", true; "legacy")]
#[test_case("\n {\"accounts\": {}, \"version\": 2} \n", false; "current")]
#[test_case("", false; "zero bytes")]
#[test_case(" \n\t", false; "whitespace")]
#[tokio::test]
pub async fn test_json_migration(original: &str, migrated: bool) {
    let directory = tempdir().unwrap();
    let path = directory.path().join("accounts.json");
    fs::write(&path, original).unwrap();
    let output = runner(&[
        "--json",
        "--accounts-file",
        "accounts.json",
        "account",
        "migrate",
    ])
    .current_dir(directory.path())
    .assert()
    .success();
    let response: Value = serde_json::from_slice(&output.get_output().stdout).unwrap();
    let outcome = if migrated {
        json!({"Performed": {"from": "v1", "to": "v2", "backup_path": "accounts.json.v1.bak"}})
    } else {
        json!({"NotRequired": {"version": "v2"}})
    };
    assert_eq!(
        response,
        json!({
            "command": "account migrate", "type": "response", "migration_outcome": outcome
        })
    );
    let backup = directory.path().join("accounts.json.v1.bak");
    if migrated {
        assert_eq!(fs::read_to_string(backup).unwrap(), original);
    } else {
        assert!(!backup.exists());
    }
    if original.contains("\"version\"") {
        assert_eq!(fs::read_to_string(&path).unwrap(), original);
    } else {
        let contents: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(contents, json!({"version": 2, "accounts": {}}));
    }
    if original.trim().is_empty() {
        runner(&["--accounts-file", "accounts.json", "account", "migrate"])
            .current_dir(directory.path())
            .assert()
            .success()
            .stdout_eq("Success: Accounts file uses the latest schema version v2\n");
    }
}
