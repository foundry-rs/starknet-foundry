use crate::helpers::constants::{SCRIPTS_DIR, URL};
use indoc::indoc;
use snapbox::cmd::{cargo_bin, Command};

#[tokio::test]
async fn test_with_calldata() {
    let script_name = "with_calldata";
    let args = vec![
        "--accounts-file",
        "../../accounts/accounts.json",
        "--account",
        "user4",
        "--url",
        URL,
        "script",
        &script_name,
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/deploy")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        command: script
        status: success
    "});
}


#[tokio::test]
async fn test_same_salt_and_class_hash_deployed_twice() {
    //TODO: Consider better error (ContractAddressUnavailableForDeployment)
    let script_name = "same_class_hash_and_salt";
    let args = vec![
        "--accounts-file",
        "../../accounts/accounts.json",
        "--account",
        "user3",
        "--url",
        URL,
        "script",
        &script_name,
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/deploy")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        ScriptCommandError::RPCError(RPCError::StarknetError(StarknetError::ContractAddressUnavailableForDeployment(())))
        command: script
        status: success
    "});
}

#[tokio::test]
async fn test_invalid_class_hash() {
    let script_name = "invalid_class_hash";
    let args = vec![
        "--accounts-file",
        "../../accounts/accounts.json",
        "--account",
        "user2",
        "--url",
        URL,
        "script",
        &script_name,
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/deploy")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        ScriptCommandError::RPCError(RPCError::StarknetError(StarknetError::ClassNotDeclared(())))
        command: script
        status: success
    "});
}

#[tokio::test]
async fn test_invalid_call_data() {
    let script_name = "invalid_calldata";
    let args = vec![
        "--accounts-file",
        "../../accounts/accounts.json",
        "--account",
        "user5",
        "--url",
        URL,
        "script",
        &script_name,
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/deploy")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        ScriptCommandError::RPCError(RPCError::StarknetError(StarknetError::TransactionReverted(())))
        command: script
        status: success
    "});
}

#[tokio::test]
async fn test_invalid_nonce() {
    let script_name = "invalid_nonce";
    let args = vec![
        "--accounts-file",
        "../../accounts/accounts.json",
        "--account",
        "user5",
        "--url",
        URL,
        "script",
        &script_name,
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/deploy")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        ScriptCommandError::RPCError(RPCError::StarknetError(StarknetError::InvalidTransactionNonce(())))
        command: script
        status: success
    "});
}

