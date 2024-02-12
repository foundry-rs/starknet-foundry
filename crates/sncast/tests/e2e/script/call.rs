use crate::helpers::constants::{SCRIPTS_DIR, URL};
use indoc::indoc;
use snapbox::cmd::{cargo_bin, Command};

#[tokio::test]
async fn test_happy_case() {
    let script_name = "call_happy";
    let args = vec!["--url", URL, "script", &script_name];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/misc")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        command: script
        status: success
    "});
}

#[tokio::test]
async fn test_failing() {
    let script_name = "call_fail";
    let args = vec!["--url", URL, "script", &script_name];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/misc")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        command: script
        msg:[..]
            original value: [120143725730386528430744932], converted to a string: [call failed]

        status: script panicked
    "});
}

#[tokio::test]
async fn test_call_invalid_entry_point() {
    let script_name = "invalid_entry_point";
    let args = vec!["--url", URL, "script", &script_name];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/call")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r#"
        ...
        test
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::ContractError(ErrorData { msg: "Entry point EntryPointSelector(StarkFelt([..])) not found in contract." })))
        command: script
        status: success
    "#});
}

#[tokio::test]
async fn test_call_invalid_address() {
    let script_name = "invalid_address";
    let args = vec!["--url", URL, "script", &script_name];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/call")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::ContractNotFound(())))
        command: script
        status: success
    "});
}

#[tokio::test]
async fn test_call_invalid_calldata() {
    let script_name = "invalid_calldata";
    let args = vec!["--url", URL, "script", &script_name];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/call")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r#"
        ...
        test
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::ContractError(ErrorData { msg: "Error at pc=0:1401:
        An ASSERT_EQ instruction failed: 5:2 != 5:5.
        " })))
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::ContractError(ErrorData { msg: "Error at pc=0:1401:
        An ASSERT_EQ instruction failed: 5:2 != 5:1.
        " })))
        command: script
        status: success
    "#});
}
