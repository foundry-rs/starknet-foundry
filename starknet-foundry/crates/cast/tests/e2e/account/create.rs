use crate::helpers::constants::NETWORK;
use crate::helpers::runner::runner;
use camino::Utf8PathBuf;
use indoc::indoc;
use std::fs;

#[tokio::test]
pub async fn test_happy_case_save_to_file() {
    let output_dir = Utf8PathBuf::from("./tmp");
    let output_path = output_dir.join("accounts.json");
    let args = vec![
        "--network",
        NETWORK,
        "account",
        "create",
        "--output-path",
        output_path.as_str(),
        "--name",
        "my_account",
        "--salt",
        "0x1",
        "--constructor-calldata",
        "123 321",
    ];

    let snapbox = runner(&args);

    snapbox.assert().success().stdout_matches(indoc! {r#"
        command: Create account
        message: Account successfully created. Prefund generated address with some tokens.
    "#});

    let contents = fs::read_to_string(output_path.clone()).expect("Unable to read created file");
    assert!(contents.contains("my_account"));
    assert!(contents.contains("alpha-goerli"));
    assert!(contents.contains("private_key"));
    assert!(contents.contains("public_key"));
    assert!(contents.contains("address"));
    assert!(contents.contains("salt"));

    fs::remove_dir_all(output_dir).unwrap();
}

#[tokio::test]
pub async fn test_happy_case_write_to_stdout() {
    let args = vec![
        "--network",
        NETWORK,
        "account",
        "create",
        "--name",
        "my_account",
        "--salt",
        "0x1",
        "--constructor-calldata",
        "123 321",
    ];

    let snapbox = runner(&args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");

    assert!(out.stderr.is_empty());
    assert!(stdout_str.contains("command: Create account"));
    assert!(stdout_str.contains("private_key:"));
    assert!(stdout_str.contains("public_key:"));
    assert!(stdout_str.contains("address:"));
    assert!(stdout_str.contains("salt: 0x1"));
}

#[tokio::test]
pub async fn test_output_path_without_network() {
    let output_dir = Utf8PathBuf::from("./tmp2");
    let output_path = output_dir.join("accounts.json");
    let args = vec![
        "account",
        "create",
        "--output-path",
        output_path.as_str(),
        "--name",
        "my_account",
        "--salt",
        "0x1",
        "--constructor-calldata",
        "123 321",
    ];

    let snapbox = runner(&args);

    snapbox.assert().stderr_matches(indoc! {r#"
        error: Argument `network` has to be passed when `output-path` provided
    "#});

    fs::remove_dir_all(output_dir).unwrap();
}

#[tokio::test]
pub async fn test_output_path_without_name() {
    let output_dir = Utf8PathBuf::from("./tmp3");
    let output_path = output_dir.join("accounts.json");
    let args = vec![
        "--network",
        NETWORK,
        "account",
        "create",
        "--output-path",
        output_path.as_str(),
        "--salt",
        "0x1",
        "--constructor-calldata",
        "123 321",
    ];

    let snapbox = runner(&args);

    snapbox.assert().stderr_matches(indoc! {r#"
        error: Argument `name` has to be passed when `output-path` provided
    "#});

    fs::remove_dir_all(output_dir).unwrap();
}

#[tokio::test]
pub async fn test_account_already_exists() {
    let output_path = Utf8PathBuf::from("./tests/data/accounts/accounts.json");
    let args = vec![
        "--network",
        NETWORK,
        "account",
        "create",
        "--output-path",
        output_path.as_str(),
        "--name",
        "user1",
        "--salt",
        "0x1",
        "--constructor-calldata",
        "123 321",
    ];

    let snapbox = runner(&args);

    snapbox.assert().stderr_matches(indoc! {r#"
        error: Account with provided name already exists in this network
    "#});
}
