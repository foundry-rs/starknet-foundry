use crate::helpers::constants::{SCRIPTS_DIR, URL};
use indoc::indoc;
use snapbox::cmd::{cargo_bin, Command};

#[tokio::test]
async fn test_call_invalid_entry_point() {
    //TODO: Consider better error
    let script_name = "invalid_entry_point";
    let args = vec!["--url", URL, "script", &script_name];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/error_handling/call")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        ScriptCommandError::RPCError(RPCError::StarknetError(StarknetError::ContractError(())))
        command: script
        status: success
    "});
}

#[tokio::test]
async fn test_call_invalid_address() {
    let script_name = "invalid_address";
    let args = vec!["--url", URL, "script", &script_name];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/error_handling/call")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        [DEBUG]	0x436f6e74726163744e6f74466f756e64 ('ContractNotFound')
        command: script
        status: success
    "});
}

#[tokio::test]
async fn test_call_invalid_calldata() {
    //TODO: Consider better error
    let script_name = "invalid_calldata";
    let args = vec!["--url", URL, "script", &script_name];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/error_handling/call")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        ScriptCommandError::RPCError(RPCError::StarknetError(StarknetError::ContractError(())))
        ScriptCommandError::RPCError(RPCError::StarknetError(StarknetError::ContractError(())))
        command: script
        status: success
    "});
}

#[tokio::test]
async fn test_declare_wrong_contract_name() {
    let script_name = "declare_missing_contract";
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
        .current_dir(SCRIPTS_DIR.to_owned() + "/error_handling/declare")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        [DEBUG]	0x436f6e74726163744172746966616374734e6f74466f756e64 ('ContractArtifactsNotFound')
        command: script
        status: success
    "});
}

#[tokio::test]
async fn test_declare_same_contract_twice() {
    let script_name = "declare_same_contract_twice";
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
        .current_dir(SCRIPTS_DIR.to_owned() + "/error_handling/declare")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        [DEBUG]	0x73756363657373 ('success')
        [DEBUG]	0x436c617373416c72656164794465636c61726564 ('ClassAlreadyDeclared')
        command: script
        status: success
    "});
}

#[tokio::test]
async fn test_declare_with_invalid_max_fee() {
    let script_name = "declare_with_invalid_max_fee";
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
        .current_dir(SCRIPTS_DIR.to_owned() + "/error_handling/declare")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        ScriptCommandError::RPCError(RPCError::StarknetError(StarknetError::InsufficientMaxFee(())))
        command: script
        status: success
    "});
}

#[tokio::test]
async fn test_declare_with_invalid_nonce() {
    let script_name = "declare_with_invalid_nonce";
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
        .current_dir(SCRIPTS_DIR.to_owned() + "/error_handling/declare")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        [DEBUG]	0x496e76616c69645472616e73616374696f6e4e6f6e6365 ('InvalidTransactionNonce')
        command: script
        status: success
    "});
}

#[tokio::test]
async fn test_declare_insufficient_account_balance() {
    let script_name = "declare_insufficient_account_balance";
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
        .current_dir(SCRIPTS_DIR.to_owned() + "/error_handling/declare")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        [DEBUG]	0x496e73756666696369656e744163636f756e7442616c616e6365 ('InsufficientAccountBalance')
        command: script
        status: success
    "});
}

#[tokio::test]
async fn test_deploy_same_salt_and_class_hash_deployed_twice() {
    //TODO: Consider better error (ContractAddressUnavailableForDeployment)
    let script_name = "deploy_same_class_hash_and_salt";
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
        .current_dir(SCRIPTS_DIR.to_owned() + "/error_handling/deploy")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        [DEBUG]	0x41646472556e617661696c61626c65466f724465706c6f796d656e74 ('AddrUnavailableForDeployment')
        command: script
        status: success
    "});
}

#[tokio::test]
async fn test_deploy_invalid_class_hash() {
    let script_name = "deploy_invalid_class_hash";
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
        .current_dir(SCRIPTS_DIR.to_owned() + "/error_handling/deploy")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        [DEBUG]	0x436c6173734e6f744465636c61726564 ('ClassNotDeclared')
        command: script
        status: success
    "});
}

#[tokio::test]
async fn test_deploy_invalid_call_data() {
    let script_name = "deploy_invalid_calldata";
    let args = vec![
        "--accounts-file",
        "../../../accounts/accounts.json",
        "--account",
        "user5",
        "--url",
        URL,
        "script",
        &script_name,
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/error_handling/deploy")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        [DEBUG]	0x5472616e73616374696f6e5265766572746564 ('TransactionReverted')
        command: script
        status: success
    "});
}

#[tokio::test]
async fn test_deploy_invalid_nonce() {
    let script_name = "deploy_invalid_nonce";
    let args = vec![
        "--accounts-file",
        "../../../accounts/accounts.json",
        "--account",
        "user5",
        "--url",
        URL,
        "script",
        &script_name,
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/error_handling/deploy")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        [DEBUG]	0x496e76616c69645472616e73616374696f6e4e6f6e6365 ('InvalidTransactionNonce')
        command: script
        status: success
    "});
}
