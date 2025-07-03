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
        Success: Script execution completed

        Status: success
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
        Success: Script execution completed

        Status: script panicked


            0x63616c6c206661696c6564 ('call failed')
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
        Success: Script execution completed
        
        Status: success
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
        Success: Script execution completed

        Status: success
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

    // TODO(#3116): Change message to string after issue with undecoded felt is resolved.
    assert_stdout_contains(
        output,
        indoc! {r#"
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::ContractError(ContractErrorData { revert_error: ContractExecutionError::Nested(&ContractExecutionErrorInner { [..] error: ContractExecutionError::Message("["0x496e70757420746f6f206c6f6e6720666f7220617267756d656e7473"]") }) })))
        ScriptCommandError::ProviderError(ProviderError::StarknetError(StarknetError::ContractError(ContractErrorData { revert_error: ContractExecutionError::Nested(&ContractExecutionErrorInner { [..] error: ContractExecutionError::Message("["0x4661696c656420746f20646573657269616c697a6520706172616d202332"]") }) })))
        Success: Script execution completed
        
        Status: success
        "#},
    );
}
