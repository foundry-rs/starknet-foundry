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
        ScriptCommandError::RPCError(RPCError::StarknetError(StarknetError::TransactionReverted(ErrorData { msg: "Error in the called contract (0x04f740e090e9930518a3a7efc76c6a61b9dffa09dfb20aae645645af739cfac8):
        Error at pc=0:81:
        Got an exception while executing a hint.
        Cairo traceback (most recent call last):
        Unknown location (pc=0:731)
        Unknown location (pc=0:677)
        Unknown location (pc=0:291)
        Unknown location (pc=0:314)

        Error in the called contract (0x041a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02bf):
        Error at pc=0:32:
        Got an exception while executing a hint: Custom Hint Error: Requested ContractAddress(PatriciaKey(StarkFelt("0x05c6341b6bab94b34abd6ad24f232d4b05b5990c7d5fa06de4de6910d2352d4c"))) is unavailable for deployment.
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
        ScriptCommandError::RPCError(RPCError::StarknetError(StarknetError::TransactionReverted(ErrorData { msg: "Error in the called contract (0x03e40c4c2770812f69166a12b0462e887ecf58a2eba5b7be1fba78450fd07dbd):
        Error at pc=0:81:
        Got an exception while executing a hint.
        Cairo traceback (most recent call last):
        Unknown location (pc=0:731)
        Unknown location (pc=0:677)
        Unknown location (pc=0:291)
        Unknown location (pc=0:314)

        Error in the called contract (0x041a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02bf):
        Error at pc=0:32:
        Got an exception while executing a hint: Custom Hint Error: Class with hash ClassHash(
            StarkFelt(
                "0x000000000000000000000000000000000000000000000000000000000000dddd",
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
        ScriptCommandError::RPCError(RPCError::StarknetError(StarknetError::TransactionReverted(ErrorData { msg: "Error in the called contract (0x051b3c1e02298485322763acbe269b53f46217adc1dc66f07e3e6f6d0fe885b7):
        Error at pc=0:81:
        Got an exception while executing a hint.
        Cairo traceback (most recent call last):
        Unknown location (pc=0:731)
        Unknown location (pc=0:677)
        Unknown location (pc=0:291)
        Unknown location (pc=0:314)

        Error in the called contract (0x041a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02bf):
        Error at pc=0:32:
        Got an exception while executing a hint: Custom Hint Error: Execution failed. Failure reason: 0x4661696c656420746f20646573657269616c697a6520706172616d202332 ('Failed to deserialize param #2').
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
        ScriptCommandError::RPCError(RPCError::StarknetError(StarknetError::InvalidTransactionNonce(())))
        command: script
        status: success
    "});
}
