use crate::helpers::constants::{SCRIPTS_DIR, URL};
use crate::helpers::fixtures::{
    duplicate_contract_directory_with_salt, duplicate_script_directory, get_accounts_path,
};
use indoc::indoc;
use snapbox::cmd::{cargo_bin, Command};

#[tokio::test]
async fn test_missing_field() {
    let script_name = "missing_field";
    let args = vec![
        "--accounts-file",
        "../../../accounts/accounts.json",
        "--account",
        "user4",
        "--url",
        URL,
        "script",
        &script_name,
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/declare/missing_field")
        .args(args);
    snapbox.assert().failure().stdout_matches(indoc! {r"
        ...
        error: Wrong number of arguments. Expected 3, found: 2
        ...
    "});
}

#[tokio::test]
async fn test_wrong_contract_name() {
    let script_name = "no_contract";
    let args = vec![
        "--accounts-file",
        "../../../accounts/accounts.json",
        "--account",
        "user4",
        "--url",
        URL,
        "script",
        &script_name,
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/declare/test_scripts")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r#"
        ...
        ScriptCommandError::ContractArtifactsNotFound(ErrorData { msg: "Mapaaaa" })
        command: script
        status: success
    "#});
}

#[tokio::test]
async fn test_same_contract_twice() {
    let contract_dir = duplicate_contract_directory_with_salt(
        SCRIPTS_DIR.to_owned() + "/map_script/contracts/",
        "dummy",
        "69",
    );
    let script_dir = duplicate_script_directory(
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
        &script_name,
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(script_dir.path())
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r#"
        ...
        success
        ScriptCommandError::ProviderError(ProviderError::Other(ErrorData { msg: "JSON-RPC error: code=-1, message="Class with hash ClassHash(StarkFelt("[..]")) is already declared."" }))
        command: script
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
        &script_name,
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/declare/test_scripts")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::InsufficientMaxFee(())))
        command: script
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
        &script_name,
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/declare/test_scripts")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::InvalidTransactionNonce(())))
        command: script
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
        &script_name,
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/declare/test_scripts")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::InsufficientAccountBalance(())))
        command: script
        status: success
    "});
}
