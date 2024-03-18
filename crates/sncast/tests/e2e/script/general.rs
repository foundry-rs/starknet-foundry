use crate::helpers::constants::{ACCOUNT_FILE_PATH, SCRIPTS_DIR, URL};
use crate::helpers::fixtures::{
    copy_directory_to_tempdir, copy_script_directory_to_tempdir,
    copy_workspace_directory_to_tempdir, duplicate_contract_directory_with_salt, get_accounts_path,
};
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stderr_contains;
use tempfile::tempdir;

#[tokio::test]
async fn test_happy_case() {
    let contract_dir = duplicate_contract_directory_with_salt(
        SCRIPTS_DIR.to_owned() + "/map_script/contracts/",
        "dummy",
        "21",
    );
    let script_dir = copy_script_directory_to_tempdir(
        SCRIPTS_DIR.to_owned() + "/map_script/scripts/",
        vec![contract_dir.as_ref()],
    );

    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "map_script";
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

    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        command: script run
        status: success
    "});
}

#[tokio::test]
async fn test_run_script_from_different_directory_no_path_to_scarb_toml() {
    let tempdir = tempdir().expect("Unable to create temporary directory");
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "call_happy";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "--url",
        URL,
        "script",
        "run",
        &script_name,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        "Error: Path to Scarb.toml manifest does not exist =[..]",
    );
}

#[tokio::test]
async fn test_fail_when_using_starknet_syscall() {
    let script_dir =
        copy_script_directory_to_tempdir(SCRIPTS_DIR.to_owned() + "/misc", Vec::<String>::new());
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "using_starknet_syscall";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "--url",
        URL,
        "script",
        "run",
        &script_name,
    ];

    let snapbox = runner(&args).current_dir(script_dir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: script run
        error: Got an exception while executing a hint: Hint Error: Starknet syscalls are not supported
        "},
    );
}

#[tokio::test]
async fn test_incompatible_sncast_std_version() {
    let script_dir = copy_directory_to_tempdir(SCRIPTS_DIR.to_owned() + "/old_sncast_std/scripts");
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "map_script";
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

    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        [WARNING] Package sncast_std version does not meet the recommended version requirement =0.19.0, it might result in unexpected behaviour
        ...
    "});
}

#[tokio::test]
async fn test_multiple_packages_not_picked() {
    let workspace_dir = copy_workspace_directory_to_tempdir(
        SCRIPTS_DIR.to_owned() + "/packages",
        vec!["crates/scripts/script1", "crates/scripts/script2"],
        Vec::<String>::new().as_ref(),
    );
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "script1";
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

    let snapbox = runner(&args).current_dir(workspace_dir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        "Error: More than one package found in scarb metadata - specify package using --package flag",
    );
}

#[tokio::test]
async fn test_multiple_packages_happy_case() {
    let workspace_dir = copy_workspace_directory_to_tempdir(
        SCRIPTS_DIR.to_owned() + "/packages",
        vec!["crates/scripts/script1", "crates/scripts/script2"],
        Vec::<String>::new().as_ref(),
    );
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "script1";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user4",
        "--url",
        URL,
        "script",
        "run",
        "--package",
        &script_name,
        &script_name,
    ];

    let snapbox = runner(&args).current_dir(workspace_dir.path());

    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        command: script run
        status: success
    "});
}

#[tokio::test]
async fn test_run_script_display_debug_traits() {
    let contract_dir = duplicate_contract_directory_with_salt(
        SCRIPTS_DIR.to_owned() + "/map_script/contracts/",
        "dummy",
        "45",
    );
    let script_dir = copy_script_directory_to_tempdir(
        SCRIPTS_DIR.to_owned() + "/map_script/scripts/",
        vec![contract_dir.as_ref()],
    );

    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let script_name = "display_debug_traits_for_subcommand_responses";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user6",
        "--url",
        URL,
        "script",
        "run",
        &script_name,
    ];

    let snapbox = runner(&args).current_dir(script_dir.path());

    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        test
        declare_nonce: [..]
        debug declare_nonce: [..]
        Transaction hash = 0x[..]
        declare_result: class_hash: [..], transaction_hash: [..]
        debug declare_result: DeclareResult { class_hash: [..], transaction_hash: [..] }
        Transaction hash = 0x[..]
        deploy_result: contract_address: [..], transaction_hash: [..]
        debug deploy_result: DeployResult { contract_address: [..], transaction_hash: [..] }
        Transaction hash = 0x[..]
        invoke_result: [..]
        debug invoke_result: InvokeResult { transaction_hash: [..] }
        call_result: [2]
        debug call_result: CallResult { data: [2] }
        command: script run
        status: success
    "});
}

#[tokio::test]
async fn test_nonexistent_account_address() {
    let script_name = "map_script";
    let args = vec![
        "--accounts-file",
        "../../../accounts/faulty_accounts.json",
        "--account",
        "with_nonexistent_address",
        "--url",
        URL,
        "script",
        "run",
        &script_name,
    ];

    let snapbox = runner(&args).current_dir(SCRIPTS_DIR.to_owned() + "/map_script/scripts");
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: script run
        error: Invalid account address
        "},
    );
}

#[tokio::test]
async fn test_no_account_passed() {
    let script_name = "map_script";
    let args = vec!["--url", URL, "script", "run", &script_name];

    let snapbox = runner(&args).current_dir(SCRIPTS_DIR.to_owned() + "/map_script/scripts");
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: script run
        error: [..] Account not defined. Please ensure the correct account is passed to `script run` command
        "},
    );
}

#[tokio::test]
async fn test_missing_field() {
    let tempdir = copy_script_directory_to_tempdir(
        SCRIPTS_DIR.to_owned() + "/missing_field",
        Vec::<String>::new(),
    );
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "missing_field";
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
    snapbox.assert().failure().stdout_matches(indoc! {r"
        ...
        error: Wrong number of arguments. Expected 3, found: 2
        ...
    "});
}
