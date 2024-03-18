use crate::helpers::constants::{ACCOUNT_FILE_PATH, SCRIPTS_DIR, URL};
use crate::helpers::fixtures::{copy_script_directory_to_tempdir, get_accounts_path};
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;

#[tokio::test]
async fn test_with_calldata() {
    let tempdir =
        copy_script_directory_to_tempdir(SCRIPTS_DIR.to_owned() + "/deploy", Vec::<String>::new());
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "with_calldata";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user4",
        "--url",
        URL,
        "script",
        "run",
        &script_name,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        command: script run
        status: success
        "},
    );
}

#[tokio::test]
async fn test_same_salt_and_class_hash_deployed_twice() {
    let tempdir =
        copy_script_directory_to_tempdir(SCRIPTS_DIR.to_owned() + "/deploy", Vec::<String>::new());
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "same_class_hash_and_salt";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user3",
        "--url",
        URL,
        "script",
        "run",
        &script_name,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r#"
        ScriptCommandError::WaitForTransactionError(WaitForTransactionError::TransactionError(TransactionError::Reverted(ErrorData { msg: "Error in the called contract ([..]):
        Error at pc=0:81:
        Got an exception while executing a hint.
        Cairo traceback (most recent call last):
        Unknown location (pc=0:731)
        Unknown location (pc=0:677)
        Unknown location (pc=0:291)
        Unknown location (pc=0:314)
        Error in the called contract ([..]):
        Error at pc=0:32:
        Transaction hash = [..]
        Transaction hash = [..]
        Got an exception while executing a hint: Requested ContractAddress(PatriciaKey(StarkFelt("[..]"))) is unavailable for deployment.
        Cairo traceback (most recent call last):
        Unknown location (pc=0:174)
        Unknown location (pc=0:127)
        " })))
        command: script run
        status: success
        "#},
    );
}

#[tokio::test]
async fn test_invalid_class_hash() {
    let tempdir =
        copy_script_directory_to_tempdir(SCRIPTS_DIR.to_owned() + "/deploy", Vec::<String>::new());
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "invalid_class_hash";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user2",
        "--url",
        URL,
        "script",
        "run",
        &script_name,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r#"
        ScriptCommandError::WaitForTransactionError(WaitForTransactionError::TransactionError(TransactionError::Reverted(ErrorData { msg: "Error in the called contract ([..]):
        Error at pc=0:81:
        Got an exception while executing a hint.
        Cairo traceback (most recent call last):
        Unknown location (pc=0:731)
        Unknown location (pc=0:677)
        Unknown location (pc=0:291)
        Unknown location (pc=0:314)
        Error in the called contract ([..]):
        Error at pc=0:32:
        Transaction hash = [..]
        Got an exception while executing a hint: Class with hash ClassHash(
            StarkFelt(
                "[..]",
            ),
        ) is not declared.
        Cairo traceback (most recent call last):
        Unknown location (pc=0:174)
        Unknown location (pc=0:127)
        " })))
        command: script run
        status: success
        "#},
    );
}

#[tokio::test]
async fn test_invalid_call_data() {
    let tempdir =
        copy_script_directory_to_tempdir(SCRIPTS_DIR.to_owned() + "/deploy", Vec::<String>::new());
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "invalid_calldata";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user5",
        "--url",
        URL,
        "script",
        "run",
        &script_name,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r#"
        ScriptCommandError::WaitForTransactionError(WaitForTransactionError::TransactionError(TransactionError::Reverted(ErrorData { msg: "Error in the called contract ([..]):
        Error at pc=0:81:
        Got an exception while executing a hint.
        Cairo traceback (most recent call last):
        Unknown location (pc=0:731)
        Unknown location (pc=0:677)
        Unknown location (pc=0:291)
        Unknown location (pc=0:314)
        Error in the called contract ([..]):
        Error at pc=0:32:
        Got an exception while executing a hint: Execution failed. Failure reason: [..] ('Failed to deserialize param #2').
        Transaction hash = [..]
        Error in the called contract ([..]):
        Execution failed. Failure reason: [..] ('Failed to deserialize param #2').
        Cairo traceback (most recent call last):
        Unknown location (pc=0:174)
        Unknown location (pc=0:127)
        " })))
        command: script run
        status: success
        "#},
    );
}

#[tokio::test]
async fn test_invalid_nonce() {
    let tempdir =
        copy_script_directory_to_tempdir(SCRIPTS_DIR.to_owned() + "/deploy", Vec::<String>::new());
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "invalid_nonce";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user5",
        "--url",
        URL,
        "script",
        "run",
        &script_name,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::InvalidTransactionNonce(())))
        command: script run
        status: success
        "},
    );
}
