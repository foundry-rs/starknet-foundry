use crate::helpers::constants::{
    ACCOUNT, ACCOUNT_FILE_PATH, CONTRACTS_DIR, MAP_CONTRACT_ADDRESS_SEPOLIA, URL,
};
use crate::helpers::fixtures::{
    copy_directory_to_tempdir, duplicate_contract_directory_with_salt, get_accounts_path,
};
use crate::helpers::runner::runner;
use configuration::test_utils::copy_config_to_tempdir;
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};

#[tokio::test]
async fn test_happy_case_from_sncast_config() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_correct.toml", None);
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "call",
        "--url",
        URL,
        "--contract-address",
        "0x0",
        "--function",
        "doesnotmatter",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        "Error: An error occurred in the called contract[..]Requested contract address [..] is not deployed[..]",
    );
}

#[tokio::test]
async fn test_happy_case_predefined_network() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_correct.toml", None);
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--profile",
        "no_url",
        "call",
        "--network",
        "sepolia",
        "--contract-address",
        "0x0",
        "--function",
        "doesnotmatter",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        "Error: An error occurred in the called contract[..]Requested contract address [..] is not deployed[..]",
    );
}

#[tokio::test]
async fn test_url_with_network_args() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_correct.toml", None);
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--profile",
        "no_url",
        "call",
        "--network",
        "sepolia",
        "--url",
        URL,
        "--contract-address",
        "0x0",
        "--function",
        "doesnotmatter",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        "error: the argument '--network <NETWORK>' cannot be used with '--url <URL>'",
    );
}

#[tokio::test]
async fn test_happy_case_from_cli_no_scarb() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--account",
        ACCOUNT,
        "call",
        "--url",
        URL,
        "--contract-address",
        "0x0",
        "--function",
        "doesnotmatter",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        "Error: An error occurred in the called contract[..]Requested contract address [..] is not deployed[..]",
    );
}

#[tokio::test]
async fn test_happy_case_from_cli_with_sncast_config() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_correct.toml", None);
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--profile",
        "default",
        "--account",
        ACCOUNT,
        "call",
        "--url",
        URL,
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "get",
        "--calldata",
        "0x0",
        "--block-id",
        "latest",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(indoc! {r"
        Success: Call completed

        Response:     0x0
        Response Raw: [0x0]
    "});
}

#[tokio::test]
async fn test_happy_case_mixed() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_correct.toml", None);
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--account",
        ACCOUNT,
        "call",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "get",
        "--calldata",
        "0x0",
        "--block-id",
        "latest",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(indoc! {r"
        Success: Call completed

        Response:     0x0
        Response Raw: [0x0]
    "});
}

#[tokio::test]
async fn test_nonexistent_account_address() {
    let contract_path =
        duplicate_contract_directory_with_salt(CONTRACTS_DIR.to_string() + "/map", "dummy", "101");
    let accounts_json_path = get_accounts_path("tests/data/accounts/faulty_accounts.json");
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "with_nonexistent_address",
        "declare",
        "--url",
        URL,
        "--contract-name",
        "Map",
    ];

    let snapbox = runner(&args).current_dir(contract_path.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        "Error: Account with address 0x1010101010011aaabbcc not found on network SN_SEPOLIA",
    );
}

#[tokio::test]
async fn test_missing_account_flag() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "declare",
        "--url",
        URL,
        "--contract-name",
        "whatever",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        "Error: Account name not passed nor found in snfoundry.toml",
    );
}

#[tokio::test]
async fn test_missing_account_flag_json() {
    let args = vec![
        "--json",
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "declare",
        "--url",
        URL,
        "--contract-name",
        "whatever",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        r#"{"command":"declare","error":"Account name not passed nor found in snfoundry.toml","type":"error"}"#,
    );
}

#[test]
fn test_accepts_full_module_path_for_ambiguous_contract_name() {
    let contract_path =
        copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/duplicate_contract_name");

    let args = vec![
        "utils",
        "class-hash",
        "--contract-name",
        "duplicate_contract_name::first_contract::HelloStarknet",
    ];

    let output = runner(&args)
        .current_dir(contract_path.path())
        .assert()
        .success();

    assert_stdout_contains(output, indoc! {r"Class Hash: 0x0[..]"});
}
