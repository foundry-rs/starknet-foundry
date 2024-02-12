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
    snapbox.assert().success().stdout_matches(indoc! {r#"
        ...
        test
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::TransactionReverted(ErrorData { msg: "Error in the called contract ([..]):
        Error at pc=0:81:
        Got an exception while executing a hint.
        Cairo traceback (most recent call last):
        Unknown location (pc=0:731)
        Unknown location (pc=0:677)
        Unknown location (pc=0:291)
        Unknown location (pc=0:314)

        Error in the called contract ([..]):
        Error at pc=0:32:
        Got an exception while executing a hint: Custom Hint Error: Requested ContractAddress(PatriciaKey(StarkFelt("[..]"))) is unavailable for deployment.
        Cairo traceback (most recent call last):
        Unknown location (pc=0:174)
        Unknown location (pc=0:127)
        " })))
        command: script
        status: success
    "#});
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
    snapbox.assert().success().stdout_matches(indoc! {r#"
        ...
        test
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::TransactionReverted(ErrorData { msg: "Error in the called contract ([..]):
        Error at pc=0:81:
        Got an exception while executing a hint.
        Cairo traceback (most recent call last):
        Unknown location (pc=0:731)
        Unknown location (pc=0:677)
        Unknown location (pc=0:291)
        Unknown location (pc=0:314)

        Error in the called contract ([..]):
        Error at pc=0:32:
        Got an exception while executing a hint: Custom Hint Error: Class with hash ClassHash(
            StarkFelt(
                "[..]",
            ),
        ) is not declared.
        Cairo traceback (most recent call last):
        Unknown location (pc=0:174)
        Unknown location (pc=0:127)
        " })))
        command: script
        status: success
    "#});
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
    snapbox.assert().success().stdout_matches(indoc! {r#"
        ...
        test
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::TransactionReverted(ErrorData { msg: "Error in the called contract ([..]):
        Error at pc=0:81:
        Got an exception while executing a hint.
        Cairo traceback (most recent call last):
        Unknown location (pc=0:731)
        Unknown location (pc=0:677)
        Unknown location (pc=0:291)
        Unknown location (pc=0:314)

        Error in the called contract ([..]):
        Error at pc=0:32:
        Got an exception while executing a hint: Custom Hint Error: Execution failed. Failure reason: [..] ('Failed to deserialize param #2').
        Cairo traceback (most recent call last):
        Unknown location (pc=0:174)
        Unknown location (pc=0:127)
        " })))
        command: script
        status: success
    "#});
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
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::InvalidTransactionNonce(())))
        command: script
        status: success
    "});
}
