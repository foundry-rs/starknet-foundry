use crate::helpers::constants::{ACCOUNT_FILE_PATH, SCRIPTS_DIR, URL};
use crate::helpers::fixtures::{
    assert_tx_entry_failed, assert_tx_entry_success, copy_directory_to_tempdir,
    copy_script_directory_to_tempdir, copy_workspace_directory_to_tempdir,
    duplicate_contract_directory_with_salt, get_accounts_path,
};
use crate::helpers::runner::runner;
use camino::Utf8PathBuf;
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};
use sncast::get_default_state_file_name;
use sncast::state::state_file::{read_txs_from_state_file, ScriptTransactionStatus};
use tempfile::tempdir;
use test_case::test_case;

#[test_case("cairo0"; "cairo_0_account")]
#[test_case("cairo1"; "cairo_1_account")]
#[tokio::test]
async fn test_happy_case(account: &str) {
    let contract_dir = duplicate_contract_directory_with_salt(
        SCRIPTS_DIR.to_owned() + "/map_script/contracts/",
        "dummy",
        account,
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
        account,
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
        [WARNING] Package sncast_std version does not meet the recommended version requirement =0.21.0, it might result in unexpected behaviour
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
        error: Account with address 0x1010101010011aaabbcc not found on network SN_SEPOLIA
        "},
    );
}

#[tokio::test]
async fn test_no_account_passed() {
    let script_name = "map_script";
    let args = vec!["--url", URL, "script", "run", &script_name];

    let snapbox = runner(&args).current_dir(SCRIPTS_DIR.to_owned() + "/map_script/scripts");
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r#"
        command: script run
        message: 
            "Account not defined. Please ensure the correct account is passed to `script run` command"
        "#},
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

#[tokio::test]
async fn test_run_script_twice_with_state_file_enabled() {
    let contract_dir = duplicate_contract_directory_with_salt(
        SCRIPTS_DIR.to_owned() + "/state_script/contracts/",
        "dummy",
        "34547",
    );
    let script_dir = copy_script_directory_to_tempdir(
        SCRIPTS_DIR.to_owned() + "/state_script/scripts/",
        vec![contract_dir.as_ref()],
    );

    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "state_script";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user7",
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

    let state_file_path = Utf8PathBuf::from_path_buf(
        script_dir
            .path()
            .join(get_default_state_file_name(script_name, "alpha-sepolia")),
    )
    .unwrap();
    let tx_entries_after_first_run = read_txs_from_state_file(&state_file_path).unwrap().unwrap();

    assert!(tx_entries_after_first_run
        .transactions
        .iter()
        .all(|(_, value)| value.status == ScriptTransactionStatus::Success));

    assert_eq!(tx_entries_after_first_run.transactions.len(), 3);

    let snapbox = runner(&args).current_dir(script_dir.path());

    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        command: script run
        status: success
    "});

    let tx_entries_after_second_run = read_txs_from_state_file(&state_file_path).unwrap().unwrap();

    assert_eq!(tx_entries_after_first_run, tx_entries_after_second_run);
}

#[tokio::test]
async fn test_state_file_contains_all_failed_txs() {
    let script_dir = copy_script_directory_to_tempdir(
        SCRIPTS_DIR.to_owned() + "/state_file/",
        Vec::<String>::new(),
    );

    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "all_tx_fail";
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

    let state_file_path = Utf8PathBuf::from_path_buf(
        script_dir
            .path()
            .join(get_default_state_file_name(script_name, "alpha-sepolia")),
    )
    .unwrap();
    let tx_entries_after_first_run = read_txs_from_state_file(&state_file_path).unwrap().unwrap();

    assert_eq!(tx_entries_after_first_run.transactions.len(), 3);

    let declare_tx_entry = tx_entries_after_first_run
        .get("2341f038132e07bd9fa3cabf5fa0c3fde26b0fc03e7b09198dbd230e1b1e071c")
        .unwrap();
    assert_tx_entry_failed(declare_tx_entry, "declare", ScriptTransactionStatus::Error, vec!["Failed to find Not_this_time artifact in starknet_artifacts.json file. Please make sure you have specified correct package using `--package` flag and that you have enabled sierra and casm code generation in Scarb.toml."]);

    let deploy_tx_entry = tx_entries_after_first_run
        .get("2402e1bcaf641961a4e97b76cad1e91f9522e4a34e57b5f740f3ea529b853c8f")
        .unwrap();
    assert_tx_entry_failed(
        deploy_tx_entry,
        "deploy",
        ScriptTransactionStatus::Fail,
        vec!["Class with hash ClassHash", "is not declared"],
    );

    let invoke_tx_entry = tx_entries_after_first_run
        .get("9e0f8008202594e57674569610b5cd22079802b0929f570dfe118b107cb24221")
        .unwrap();
    assert_tx_entry_failed(
        invoke_tx_entry,
        "invoke",
        ScriptTransactionStatus::Fail,
        vec!["Requested contract address", "is not deployed"],
    );
}

#[tokio::test]
async fn test_state_file_rerun_failed_tx() {
    let script_dir = copy_script_directory_to_tempdir(
        SCRIPTS_DIR.to_owned() + "/state_file/",
        Vec::<String>::new(),
    );
    let script_name = "rerun_failed_tx";
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);
    let state_file_path = Utf8PathBuf::from_path_buf(
        script_dir
            .path()
            .join(get_default_state_file_name(script_name, "alpha-sepolia")),
    )
    .unwrap();

    let tx_entries_before = read_txs_from_state_file(&state_file_path).unwrap().unwrap();
    assert_eq!(tx_entries_before.transactions.len(), 1);
    let invoke_tx_entry_before = tx_entries_before
        .get("1863066e9093b13eea3a3844f28674dc8d9b7e2e49a525504133169c1d382718")
        .unwrap();
    assert_tx_entry_failed(
        invoke_tx_entry_before,
        "invoke",
        ScriptTransactionStatus::Error,
        vec!["Requested contract address", "is not deployed"],
    );

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

    let tx_entries_after_first_run = read_txs_from_state_file(&state_file_path).unwrap().unwrap();
    assert_eq!(tx_entries_after_first_run.transactions.len(), 1);

    let invoke_tx_entry = tx_entries_after_first_run
        .get("1863066e9093b13eea3a3844f28674dc8d9b7e2e49a525504133169c1d382718")
        .unwrap();
    assert_tx_entry_success(invoke_tx_entry, "invoke");
}
