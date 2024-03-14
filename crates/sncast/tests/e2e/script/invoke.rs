use crate::helpers::constants::{ACCOUNT_FILE_PATH, SCRIPTS_DIR, URL};
use crate::helpers::fixtures::{copy_script_directory_to_tempdir, get_accounts_path};
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;

#[tokio::test]
async fn test_max_fee_too_low() {
    let script_dir =
        copy_script_directory_to_tempdir(SCRIPTS_DIR.to_owned() + "/invoke", Vec::<String>::new());
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "max_fee_too_low";
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

    let snapbox = runner(&args).current_dir(script_dir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::InsufficientMaxFee(())))
        command: script run
        status: success
        "},
    );
}

#[tokio::test]
async fn test_contract_does_not_exist() {
    let script_dir =
        copy_script_directory_to_tempdir(SCRIPTS_DIR.to_owned() + "/invoke", Vec::<String>::new());
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "contract_does_not_exist";
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

    let snapbox = runner(&args).current_dir(script_dir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::ContractError(ErrorData { msg: "Error in the called contract ([..]):
        Error at pc=0:81:
        Got an exception while executing a hint: Requested contract address ContractAddress(PatriciaKey(StarkFelt("[..]"))) is not deployed.
        Cairo traceback (most recent call last):
        Unknown location (pc=0:731)
        Unknown location (pc=0:677)
        Unknown location (pc=0:291)
        Unknown location (pc=0:314)
        " })))
        command: script run
        status: success
        "#},
    );
}

#[test]
fn test_wrong_function_name() {
    let script_dir =
        copy_script_directory_to_tempdir(SCRIPTS_DIR.to_owned() + "/invoke", Vec::<String>::new());
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "wrong_function_name";
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

    let snapbox = runner(&args).current_dir(script_dir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::ContractError(ErrorData { msg: "Error in the called contract ([..]):
        Error at pc=0:81:
        Got an exception while executing a hint: Entry point EntryPointSelector(StarkFelt("[..]")) not found in contract.
        Cairo traceback (most recent call last):
        Unknown location (pc=0:731)
        Unknown location (pc=0:677)
        Unknown location (pc=0:291)
        Unknown location (pc=0:314)
        " })))

        Error in the called contract ([..]):
        Entry point EntryPointSelector(StarkFelt([..])) not found in contract.
        command: script run
        status: success
        "#},
    );
}

#[test]
fn test_wrong_calldata() {
    let script_dir =
        copy_script_directory_to_tempdir(SCRIPTS_DIR.to_owned() + "/invoke", Vec::<String>::new());
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "wrong_calldata";
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

    let snapbox = runner(&args).current_dir(script_dir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::ContractError(ErrorData { msg: "Error in the called contract ([..]):
        Error at pc=0:81:
        Got an exception while executing a hint: Execution failed. Failure reason: [..] ('Failed to deserialize param #2').
        Cairo traceback (most recent call last):
        Unknown location (pc=0:731)
        Unknown location (pc=0:677)
        Unknown location (pc=0:291)
        Unknown location (pc=0:314)
        " })))

        Error in the called contract ([..]):
        Execution failed. Failure reason: [..] ('Failed to deserialize param #2').
        command: script run
        status: success
        "#},
    );
}
