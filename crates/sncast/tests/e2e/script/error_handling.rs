use crate::helpers::constants::{SCRIPTS_DIR, URL};
use indoc::indoc;
use snapbox::cmd::{cargo_bin, Command};

#[tokio::test]
async fn test_call_invalid_entry_point() {
    let script_name = "invalid_entry_point";
    let args = vec!["--url", URL, "script", &script_name];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/error_handling/call")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        [DEBUG]	0x436f6e74726163744572726f72 ('ContractError')
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
    let script_name = "invalid_calldata";
    let args = vec!["--url", URL, "script", &script_name];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/error_handling/call")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        [DEBUG]	0x436f6e74726163744572726f72 ('ContractError')
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
        command: script
        error: Got an exception while executing a hint: [..]
    "});
}
