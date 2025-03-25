use crate::helpers::constants::{SCRIPTS_DIR, URL};
use crate::helpers::fixtures::copy_script_directory_to_tempdir;
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;

#[tokio::test]
async fn test_happy_case() {
    let tempdir =
        copy_script_directory_to_tempdir(SCRIPTS_DIR.to_owned() + "/misc", Vec::<String>::new());

    let script_name = "call_happy";
    let args = vec!["script", "run", &script_name, "--url", URL];

    let snapbox = runner(&args).current_dir(tempdir.path());
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        command: script run
        status: success
    "});
}

#[tokio::test]
async fn test_failing() {
    let tempdir =
        copy_script_directory_to_tempdir(SCRIPTS_DIR.to_owned() + "/misc", Vec::<String>::new());

    let script_name = "call_fail";
    let args = vec!["script", "run", &script_name, "--url", URL];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        command: script run
        message:[..]
            0x63616c6c206661696c6564 ('call failed')

        status: script panicked
        "},
    );
}

#[tokio::test]
async fn test_call_invalid_entry_point() {
    let tempdir =
        copy_script_directory_to_tempdir(SCRIPTS_DIR.to_owned() + "/call", Vec::<String>::new());

    let script_name = "invalid_entry_point";
    let args = vec!["script", "run", &script_name, "--url", URL];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::EntryPointNotFound(())))
        command: script run
        status: success
        "},
    );
}

#[tokio::test]
async fn test_call_invalid_address() {
    let tempdir =
        copy_script_directory_to_tempdir(SCRIPTS_DIR.to_owned() + "/call", Vec::<String>::new());

    let script_name = "invalid_address";
    let args = vec!["script", "run", &script_name, "--url", URL];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::ContractNotFound(())))
        command: script run
        status: success
        "},
    );
}

#[tokio::test]
async fn test_call_invalid_calldata() {
    let tempdir =
        copy_script_directory_to_tempdir(SCRIPTS_DIR.to_owned() + "/call", Vec::<String>::new());

    let script_name = "invalid_calldata";
    let args = vec!["script", "run", &script_name, "--url", URL];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    // TODO(#3120): Update asserted message once displaying is implemented
    assert_stdout_contains(
        output,
        indoc! {r#"
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::ContractError(ErrorData { revert_error: ContractExecutionError::Nested(&ContractExecutionErrorInner { [..] error: ContractExecutionError::Message([2, 161019049007022372777416340987812303282620498837842361643383982666764674358, 97522975666756167445258504733288874551174906205732890712698934399253815901, 0, 0]) }) })))
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::ContractError(ErrorData { revert_error: ContractExecutionError::Nested(&ContractExecutionErrorInner { [..] error: ContractExecutionError::Message([2, 161019049007017550688154859146124165449376331526496475447250082491572630326, 94023844190060481618082450560698606437386733826467150857039051259452076595, 858923613, 4]) }) })))
        command: script run
        command: script run
        status: success
        "#},
    );
}
