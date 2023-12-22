use crate::helpers::constants::{SCRIPTS_DIR, URL};
use indoc::indoc;
use snapbox::cmd::{cargo_bin, Command};

#[tokio::test]
async fn test_max_fee_too_low() {
    let script_name = "max_fee_too_low";
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
        .current_dir(SCRIPTS_DIR.to_owned() + "/invoke")
        .args(args);
    snapbox.assert().success().stderr_matches(indoc! {r"
        ...
        command: script
        error: Got an exception while executing a hint: Custom Hint Error: Max fee is smaller than the minimal transaction cost
    "});
}

#[tokio::test]
async fn test_contract_does_not_exist() {
    let script_name = "contract_does_not_exist";
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
        .current_dir(SCRIPTS_DIR.to_owned() + "/invoke")
        .args(args);
    snapbox.assert().success().stderr_matches(indoc! {r"
        ...
        command: script
        error: Got an exception while executing a hint: Custom Hint Error: An error occurred in the called contract [..]
    "});
}

#[test]
fn test_wrong_function_name() {
    let script_name = "wrong_function_name";
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
        .current_dir(SCRIPTS_DIR.to_owned() + "/invoke")
        .args(args);
    snapbox.assert().success().stderr_matches(indoc! {r"
        ...
        command: script
        error: Got an exception while executing a hint: Custom Hint Error: An error occurred in the called contract [..]
    "});
}

#[test]
fn test_wrong_calldata() {
    let script_name = "wrong_calldata";
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
        .current_dir(SCRIPTS_DIR.to_owned() + "/invoke")
        .args(args);
    snapbox.assert().success().stderr_matches(indoc! {r"
        ...
        command: script
        error: Got an exception while executing a hint: Custom Hint Error: An error occurred in the called contract [..]
    "});
}
