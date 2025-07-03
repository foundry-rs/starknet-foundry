use crate::helpers::constants::{ACCOUNT_FILE_PATH, SCRIPTS_DIR, URL};
use crate::helpers::fixtures::duplicate_contract_directory_with_salt;
use crate::helpers::fixtures::{copy_script_directory_to_tempdir, get_accounts_path};
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;
use test_case::test_case;

#[test_case("oz_cairo_0"; "cairo_0_account")]
#[test_case("oz_cairo_1"; "cairo_1_account")]
#[test_case("oz"; "oz_account")]
#[test_case("argent"; "argent_account")]
#[test_case("braavos"; "braavos_account")]
#[tokio::test]
async fn test_wrong_contract_name(account: &str) {
    let contract_dir = duplicate_contract_directory_with_salt(
        SCRIPTS_DIR.to_owned() + "/map_script/contracts/",
        "dummy",
        "609",
    );
    let tempdir = copy_script_directory_to_tempdir(
        SCRIPTS_DIR.to_owned() + "/declare/",
        vec![contract_dir.as_ref()],
    );
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "no_contract";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        account,
        "script",
        "run",
        &script_name,
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r#"
        ScriptCommandError::ContractArtifactsNotFound(ErrorData { msg: "Mapaaaa" })
        Success: Script execution completed
        
        Status: success
        "#},
    );
}

// TODO(#2912)
#[tokio::test]
#[ignore]
async fn test_same_contract_twice() {
    let contract_dir = duplicate_contract_directory_with_salt(
        SCRIPTS_DIR.to_owned() + "/map_script/contracts/",
        "dummy",
        "69",
    );
    let script_dir = copy_script_directory_to_tempdir(
        SCRIPTS_DIR.to_owned() + "/declare/",
        vec![contract_dir.as_ref()],
    );

    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let script_name = "same_contract_twice";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user4",
        "script",
        "run",
        &script_name,
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(script_dir.path());
    snapbox.assert().success().stdout_matches(indoc! {"
        ...
        success
        DeclareResult::Success(DeclareTransactionResult { class_hash: [..], transaction_hash: [..] })
        DeclareResult::AlreadyDeclared(AlreadyDeclaredResult { class_hash: [..] })
        Success: Script execution completed
        
        Status: success
    "});
}

#[tokio::test]
async fn test_with_invalid_max_fee() {
    let contract_dir = duplicate_contract_directory_with_salt(
        SCRIPTS_DIR.to_owned() + "/map_script/contracts/",
        "dummy",
        "19",
    );
    let script_dir = copy_script_directory_to_tempdir(
        SCRIPTS_DIR.to_owned() + "/declare/",
        vec![contract_dir.as_ref()],
    );
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "with_invalid_max_fee";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user2",
        "script",
        "run",
        &script_name,
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(script_dir.path());
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::InsufficientResourcesForValidate(())))
        Success: Script execution completed
        
        Status: success
    "});
}

#[tokio::test]
async fn test_with_invalid_nonce() {
    let contract_dir = duplicate_contract_directory_with_salt(
        SCRIPTS_DIR.to_owned() + "/map_script/contracts/",
        "dummy",
        "21",
    );
    let script_dir = copy_script_directory_to_tempdir(
        SCRIPTS_DIR.to_owned() + "/declare/",
        vec![contract_dir.as_ref()],
    );
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "with_invalid_nonce";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user4",
        "script",
        "run",
        &script_name,
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(script_dir.path());
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::InvalidTransactionNonce(())))
        Success: Script execution completed
        
        Status: success
    "});
}

#[tokio::test]
#[ignore = "TODO(#3091) Devnet response does not match te spec"]
async fn test_insufficient_account_balance() {
    let contract_dir = duplicate_contract_directory_with_salt(
        SCRIPTS_DIR.to_owned() + "/map_script/contracts/",
        "dummy",
        "21",
    );
    let script_dir = copy_script_directory_to_tempdir(
        SCRIPTS_DIR.to_owned() + "/declare/",
        vec![contract_dir.as_ref()],
    );
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "insufficient_account_balance";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user6",
        "script",
        "run",
        &script_name,
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(script_dir.path());
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::InsufficientAccountBalance(())))
        Success: Script execution completed
        
        Status: success
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
        SCRIPTS_DIR.to_owned() + "/declare/",
        vec![contract_dir.as_ref()],
    );

    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let script_name = "time_out";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user8",
        "--wait-timeout",
        "1",
        "--wait-retry-interval",
        "1",
        "script",
        "run",
        &script_name,
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(script_dir.path());
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        ScriptCommandError::WaitForTransactionError(WaitForTransactionError::TimedOut(()))
        Success: Script execution completed
        
        Status: success
    "});
}

#[tokio::test]
async fn test_fee_settings() {
    let contract_dir = duplicate_contract_directory_with_salt(
        SCRIPTS_DIR.to_owned() + "/map_script/contracts/",
        "dummy",
        "100",
    );
    let script_dir = copy_script_directory_to_tempdir(
        SCRIPTS_DIR.to_owned() + "/declare/",
        vec![contract_dir.as_ref()],
    );

    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let script_name = "fee_settings";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user4",
        "script",
        "run",
        "--url",
        URL,
        &script_name,
    ];

    let snapbox = runner(&args).current_dir(script_dir.path());
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        success
        Success: Script execution completed
        
        Status: success
    "});
}
