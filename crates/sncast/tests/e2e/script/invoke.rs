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
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::InsufficientMaxFee(())))
        command: script
        status: success
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
    snapbox.assert().success().stdout_matches(indoc! {r#"
        ...
        test
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::ContractError(ErrorData { msg: "Error in the called contract ([..]):
        Error at pc=0:81:
        Got an exception while executing a hint: Custom Hint Error: Requested contract address ContractAddress(PatriciaKey(StarkFelt("[..]"))) is not deployed.
        Cairo traceback (most recent call last):
        Unknown location (pc=0:731)
        Unknown location (pc=0:677)
        Unknown location (pc=0:291)
        Unknown location (pc=0:314)
        " })))
        command: script
        status: success
    "#});
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
    snapbox.assert().success().stdout_matches(indoc! {r#"
        ...
        test
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::ContractError(ErrorData { msg: "Error in the called contract ([..]):
        Error at pc=0:81:
        Got an exception while executing a hint: Custom Hint Error: Entry point EntryPointSelector(StarkFelt("[..]")) not found in contract.
        Cairo traceback (most recent call last):
        Unknown location (pc=0:731)
        Unknown location (pc=0:677)
        Unknown location (pc=0:291)
        Unknown location (pc=0:314)
        " })))
        command: script
        status: success
    "#});
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
    snapbox.assert().success().stdout_matches(indoc! {r#"
        ...
        test
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::ContractError(ErrorData { msg: "Error in the called contract ([..]):
        Error at pc=0:81:
        Got an exception while executing a hint: Custom Hint Error: Execution failed. Failure reason: [..] ('Failed to deserialize param #2').
        Cairo traceback (most recent call last):
        Unknown location (pc=0:731)
        Unknown location (pc=0:677)
        Unknown location (pc=0:291)
        Unknown location (pc=0:314)
        " })))
        command: script
        status: success
    "#});
}
