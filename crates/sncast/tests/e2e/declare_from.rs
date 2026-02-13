use crate::helpers::constants::{
    CONTRACTS_DIR, MAP_CONTRACT_CLASS_HASH_SEPOLIA, SEPOLIA_RPC_URL, URL,
};
use crate::helpers::fee::apply_test_resource_bounds_flags;
use crate::helpers::fixtures::{
    create_and_deploy_oz_account, duplicate_contract_directory_with_salt, get_accounts_path,
    join_tempdirs,
};
use crate::helpers::runner::runner;
use indoc::indoc;
use scarb_api::ScarbCommand;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};
use std::fs;
use std::process::Stdio;
use tempfile::tempdir;

#[tokio::test]
async fn test_happy_case() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let example_contract_class_hash_sepolia =
        "0x66802613e2cd02ea21430a56181d9ee83c54d4ccdc45efa497d41fe1dc55a0e";

    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "declare-from",
        "--class-hash",
        example_contract_class_hash_sepolia,
        "--source-url",
        SEPOLIA_RPC_URL,
        "--url",
        URL,
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        Success: Declaration completed

        Class Hash:       0x66802613e2cd02ea21430a56181d9ee83c54d4ccdc45efa497d41fe1dc55a0e
        Transaction Hash: 0x[..]
        
        To see declaration details, visit:
        class: https://[..]
        transaction: https://[..]
    " },
    );
}

#[tokio::test]
async fn test_happy_case_with_block_id() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let example_b_contract_class_hash_sepolia =
        "0x3de1a95e27b385c882c79355ca415915989e71f67c0f6f8ce146d4bcee7163c";

    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user2",
        "declare-from",
        "--class-hash",
        example_b_contract_class_hash_sepolia,
        "--source-url",
        SEPOLIA_RPC_URL,
        "--url",
        URL,
        "--block-id",
        "latest",
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        Success: Declaration completed

        Class Hash:       0x3de1a95e27b385c882c79355ca415915989e71f67c0f6f8ce146d4bcee7163c
        Transaction Hash: 0x[..]
        
        To see declaration details, visit:
        class: https://[..]
        transaction: https://[..]
    " },
    );
}

#[tokio::test]
async fn test_contract_already_declared() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user3",
        "declare-from",
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--source-url",
        SEPOLIA_RPC_URL,
        "--url",
        URL,
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: declare-from
        Error: Contract with class hash 0x0[..] is already declared
        "},
    );
}

#[tokio::test]
async fn test_class_hash_does_not_exist_on_source_network() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "declare-from",
        "--class-hash",
        "0x1",
        "--source-url",
        SEPOLIA_RPC_URL,
        "--url",
        URL,
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: declare-from
        Error: Provided class hash does not exist
        "},
    );
}

#[tokio::test]
async fn test_source_rpc_args_not_passed() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "declare-from",
        "--class-hash",
        "0x1",
        "--url",
        URL,
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        Error: Either `--source-network` or `--source-url` must be provided
        "},
    );
}

#[tokio::test]
async fn test_invalid_block_id() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "declare-from",
        "--class-hash",
        "0x1",
        "--url",
        URL,
        "--block-id",
        "0x10101",
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        Error: Either `--source-network` or `--source-url` must be provided
        "},
    );
}

#[tokio::test]
async fn test_declare_from_sierra_happy_case() {
    let contract_path = duplicate_contract_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/map",
        "put",
        "declare_from_file_happy",
    );

    let tempdir = create_and_deploy_oz_account().await;
    join_tempdirs(&contract_path, &tempdir);

    let build_output = ScarbCommand::new()
        .arg("build")
        .current_dir(tempdir.path())
        .command()
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()
        .expect("Failed to run `scarb build`");

    assert!(build_output.status.success(), "`scarb build` failed");
    let sierra_path = tempdir
        .path()
        .join("target/dev/map_Map.contract_class.json");
    assert!(
        sierra_path.exists(),
        "sierra artifact not found at {sierra_path:?}"
    );
    let sierra_path = sierra_path.to_str().unwrap();

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "declare-from",
        "--sierra-file",
        sierra_path,
        "--url",
        URL,
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        Success: Declaration completed

        Class Hash:       0x[..]
        Transaction Hash: 0x[..]

        To see declaration details, visit:
        class: https://[..]
        transaction: https://[..]
    "},
    );
}

#[tokio::test]
async fn test_declare_from_sierra_does_not_exist() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "declare-from",
        "--sierra-file",
        "/nonexistent/path/contract.contract_class.json",
        "--url",
        URL,
    ];
    let args = apply_test_resource_bounds_flags(args);
    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: declare-from
        Error: Failed to read sierra file at [..]contract_class.json: No such file or directory [..]
        "},
    );
}

#[tokio::test]
async fn test_declare_from_sierra_invalid_json() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");
    let invalid_sierra_path = temp_dir.path().join("invalid_contract_class.json");
    fs::write(&invalid_sierra_path, r#"{"not": "a valid sierra"}"#).unwrap();
    let invalid_sierra_path = invalid_sierra_path.to_str().unwrap().to_string();

    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "declare-from",
        "--sierra-file",
        invalid_sierra_path.as_str(),
        "--url",
        URL,
    ];
    let args = apply_test_resource_bounds_flags(args);
    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: declare-from
        Error: Failed to parse sierra file: missing field `sierra_program` at line 1 column 25
        "},
    );
}

#[tokio::test]
async fn test_declare_from_sierra_already_declared() {
    let contract_path = duplicate_contract_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/map",
        "put",
        "declare_from_file_already_declared",
    );
    let tempdir = create_and_deploy_oz_account().await;
    join_tempdirs(&contract_path, &tempdir);

    let build_output = ScarbCommand::new()
        .arg("build")
        .current_dir(tempdir.path())
        .command()
        .output()
        .expect("Failed to run `scarb build`");
    assert!(build_output.status.success(), "`scarb build` failed");

    let sierra_path = tempdir
        .path()
        .join("target/dev/map_Map.contract_class.json");
    let sierra_path = sierra_path.to_str().unwrap();

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "declare-from",
        "--sierra-file",
        sierra_path,
        "--url",
        URL,
    ];
    let args = apply_test_resource_bounds_flags(args);

    runner(&args).current_dir(tempdir.path()).assert().success();

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "declare-from",
        "--sierra-file",
        sierra_path,
        "--url",
        URL,
    ];
    let args = apply_test_resource_bounds_flags(args);
    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: declare-from
        Error: Contract with class hash 0x0[..] is already declared
        "},
    );
}

#[test]
fn test_declare_from_requires_sierra_file_or_class_hash() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let args = vec!["declare-from", "--url", URL];
    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        error: the following required arguments were not provided:
          <--sierra-file <SIERRA_FILE>|--class-hash <CLASS_HASH>>

        Usage: sncast declare-from --url <URL> <--sierra-file <SIERRA_FILE>|--class-hash <CLASS_HASH>>

        For more information, try '--help'.
        "},
    );
}

#[test]
fn test_declare_from_conflicting_contract_source() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let args = vec![
        "declare-from",
        "--class-hash",
        "0x1",
        "--sierra-file",
        "path/to/sierra.json",
        "--url",
        URL,
    ];
    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        error: the argument '--class-hash <CLASS_HASH>' cannot be used with '--sierra-file <SIERRA_FILE>'
        
        Usage: sncast declare-from --url <URL> <--sierra-file <SIERRA_FILE>|--class-hash <CLASS_HASH>>

        For more information, try '--help'.
        "},
    );
}
