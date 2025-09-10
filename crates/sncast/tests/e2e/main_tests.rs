use crate::helpers::constants::{
    ACCOUNT, ACCOUNT_FILE_PATH, CONTRACTS_DIR, MAP_CONTRACT_ADDRESS_SEPOLIA, URL,
};
use crate::helpers::env::set_keystore_password_env;
use crate::helpers::fee::apply_test_resource_bounds_flags;
use crate::helpers::fixtures::{
    duplicate_contract_directory_with_salt, get_accounts_path, get_keystores_path,
};
use crate::helpers::runner::runner;
use configuration::copy_config_to_tempdir;
use indoc::indoc;
use shared::test_utils::output_assert::assert_stderr_contains;

#[tokio::test]
async fn test_happy_case_from_sncast_config() {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None).unwrap();
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
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None).unwrap();
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
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None).unwrap();
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
async fn test_network_with_url_defined_in_config_toml() {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None).unwrap();
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "--profile",
        "default",
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
        "Error: The argument '--network' cannot be used when `url` is defined in `snfoundry.toml` for the active profile",
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
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None).unwrap();
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
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None).unwrap();
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
async fn test_inexistent_keystore() {
    let args = vec![
        "--keystore",
        "inexistent_key.json",
        "declare",
        "--url",
        URL,
        "--contract-name",
        "my_contract",
    ];

    let snapbox = runner(&args);

    let output = snapbox.assert().failure();
    assert_stderr_contains(output, "Error: Failed to find keystore file");
}

#[tokio::test]
async fn test_keystore_account_required() {
    let args = vec![
        "--keystore",
        "tests/data/keystore/my_key.json",
        "declare",
        "--url",
        URL,
        "--contract-name",
        "my_contract",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        "Error: Argument `--account` must be passed and be a path when using `--keystore`",
    );
}

#[tokio::test]
async fn test_keystore_inexistent_account() {
    let args = vec![
        "--keystore",
        "tests/data/keystore/my_key.json",
        "--account",
        "inexistent_account",
        "declare",
        "--url",
        URL,
        "--contract-name",
        "my_contract",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        "Error: File containing the account does not exist[..]",
    );
}

#[tokio::test]
async fn test_keystore_undeployed_account() {
    let contract_path =
        duplicate_contract_directory_with_salt(CONTRACTS_DIR.to_string() + "/map", "put", "8");
    let my_key_path = get_keystores_path("tests/data/keystore/my_key.json");
    let my_account_undeployed_path =
        get_keystores_path("tests/data/keystore/my_account_undeployed.json");

    let args = vec![
        "--keystore",
        my_key_path.as_str(),
        "--account",
        my_account_undeployed_path.as_str(),
        "declare",
        "--url",
        URL,
        "--contract-name",
        "Map",
    ];

    set_keystore_password_env();
    let snapbox = runner(&args).current_dir(contract_path.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(output, "Error: [..] make sure the account is deployed");
}

#[tokio::test]
async fn test_keystore_declare() {
    let contract_path =
        duplicate_contract_directory_with_salt(CONTRACTS_DIR.to_string() + "/map", "put", "999");
    let my_key_path = get_keystores_path("tests/data/keystore/predeployed_key.json");
    let my_account_path = get_keystores_path("tests/data/keystore/predeployed_account.json");
    let args = vec![
        "--keystore",
        my_key_path.as_str(),
        "--account",
        my_account_path.as_str(),
        "declare",
        "--url",
        URL,
        "--contract-name",
        "Map",
    ];
    let args = apply_test_resource_bounds_flags(args);

    set_keystore_password_env();
    let snapbox = runner(&args).current_dir(contract_path.path());

    assert!(snapbox.assert().success().get_output().stderr.is_empty());
}
