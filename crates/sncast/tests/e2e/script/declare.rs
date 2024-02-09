use crate::helpers::constants::{SCRIPTS_DIR, URL};
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
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        ScriptCommandError::ContractArtifactsNotFound(())
        command: script
        status: success
    "});
}

#[tokio::test]
async fn test_same_contract_twice() {
    let script_name = "same_contract_twice";
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
        ScriptCommandError::RPCError(RPCError::StarknetError(StarknetError::ClassAlreadyDeclared(())))
        command: script
        status: success
    "});
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
        ScriptCommandError::RPCError(RPCError::StarknetError(StarknetError::InsufficientMaxFee(())))
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
        ScriptCommandError::RPCError(RPCError::StarknetError(StarknetError::InvalidTransactionNonce(())))
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
        ScriptCommandError::RPCError(RPCError::StarknetError(StarknetError::InsufficientAccountBalance(())))
        command: script
        status: success
    "});
}

