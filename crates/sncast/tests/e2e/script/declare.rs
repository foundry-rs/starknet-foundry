use crate::helpers::constants::{ACCOUNT_FILE_PATH, SCRIPTS_DIR, URL};
use crate::helpers::fixtures::duplicate_contract_directory_with_salt;
use crate::helpers::fixtures::{copy_script_directory_to_tempdir, get_accounts_path};
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stderr_contains;

#[tokio::test]
async fn test_missing_field() {
    let tempdir = copy_script_directory_to_tempdir(
        SCRIPTS_DIR.to_owned() + "/declare/missing_field",
        Vec::<String>::new(),
    );
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "missing_field";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user4",
        "--url",
        URL,
        "script",
        "run",
        &script_name,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    snapbox.assert().failure().stdout_matches(indoc! {r"
        ...
        error: Wrong number of arguments. Expected 3, found: 2
        ...
    "});
}

#[tokio::test]
async fn test_wrong_contract_name() {
    let tempdir = copy_script_directory_to_tempdir(
        SCRIPTS_DIR.to_owned() + "/declare/test_scripts",
        Vec::<String>::new(),
    );
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "no_contract";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user4",
        "--url",
        URL,
        "script",
        "run",
        &script_name,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r#"
        ScriptCommandError::ContractArtifactsNotFound(ErrorData { msg: "Mapaaaa" })
        command: script run
        status: success
        "#},
    );
}

#[tokio::test]
async fn test_same_contract_twice() {
    let contract_dir = duplicate_contract_directory_with_salt(
        SCRIPTS_DIR.to_owned() + "/map_script/contracts/",
        "dummy",
        "69",
    );
    let script_dir = copy_script_directory_to_tempdir(
        SCRIPTS_DIR.to_owned() + "/declare/test_scripts",
        vec![contract_dir.as_ref()],
    );

    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let script_name = "same_contract_twice";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user4",
        "--url",
        URL,
        "script",
        "run",
        &script_name,
    ];

    let snapbox = runner(&args).current_dir(script_dir.path());
    snapbox.assert().success().stdout_matches(indoc! {r#"
        ...
        success
        ScriptCommandError::ProviderError(ProviderError::UnknownError(ErrorData { msg: "JSON-RPC error: code=-1, message="Class with hash ClassHash(StarkFelt("[..]")) is already declared."" }))
        command: script run
        status: success
    "#});
}

#[tokio::test]
async fn test_with_invalid_max_fee() {
    let script_name = "with_invalid_max_fee";
    let args = vec![
        "--accounts-file",
        "../../../accounts/accounts.json",
        "--account",
        "user2",
        "--url",
        URL,
        "script",
        "run",
        &script_name,
    ];

    let snapbox = runner(&args).current_dir(SCRIPTS_DIR.to_owned() + "/declare/test_scripts");
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::InsufficientMaxFee(())))
        command: script run
        status: success
    "});
}

#[tokio::test]
async fn test_with_invalid_nonce() {
    let script_name = "with_invalid_nonce";
    let args = vec![
        "--accounts-file",
        "../../../accounts/accounts.json",
        "--account",
        "user4",
        "--url",
        URL,
        "script",
        "run",
        &script_name,
    ];

    let snapbox = runner(&args).current_dir(SCRIPTS_DIR.to_owned() + "/declare/test_scripts");
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::InvalidTransactionNonce(())))
        command: script run
        status: success
    "});
}

#[tokio::test]
async fn test_insufficient_account_balance() {
    let script_name = "insufficient_account_balance";
    let args = vec![
        "--accounts-file",
        "../../../accounts/accounts.json",
        "--account",
        "user6",
        "--url",
        URL,
        "script",
        "run",
        &script_name,
    ];

    let snapbox = runner(&args).current_dir(SCRIPTS_DIR.to_owned() + "/declare/test_scripts");
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::InsufficientAccountBalance(())))
        command: script run
        status: success
    "});
}

#[tokio::test]
async fn test_sncast_timed_out() {
    let contract_dir = duplicate_contract_directory_with_salt(
        SCRIPTS_DIR.to_owned() + "/map_script/contracts/",
        "dummy",
        "78",
    );
    let script_dir = copy_script_directory_to_tempdir(
        SCRIPTS_DIR.to_owned() + "/declare/test_scripts",
        vec![contract_dir.as_ref()],
    );

    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let script_name = "time_out";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user8",
        "--url",
        URL,
        "--wait-timeout",
        "1",
        "--wait-retry-interval",
        "1",
        "script",
        "run",
        &script_name,
    ];

    let snapbox = runner(&args).current_dir(script_dir.path());
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        ScriptCommandError::WaitForTransactionError(WaitForTransactionError::TimedOut(()))
        command: script run
        status: success
    "});
}
