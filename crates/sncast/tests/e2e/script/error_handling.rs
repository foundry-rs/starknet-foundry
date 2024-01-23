use crate::helpers::constants::{SCRIPTS_DIR, URL};
use indoc::indoc;
use snapbox::cmd::{cargo_bin, Command};

#[tokio::test]
async fn test_call_invalid_entry_point() {
    let script_name = "invalid_entry_point";
    let args = vec!["--url", URL, "script", &script_name];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/error_handling")
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
        .current_dir(SCRIPTS_DIR.to_owned() + "/error_handling")
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
        .current_dir(SCRIPTS_DIR.to_owned() + "/error_handling")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        [DEBUG]	0x436f6e74726163744572726f72 ('ContractError')
        command: script
        status: success
    "});
}
