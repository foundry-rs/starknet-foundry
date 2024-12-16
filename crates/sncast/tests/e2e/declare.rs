use crate::helpers::constants::{CONTRACTS_DIR, DEVNET_OZ_CLASS_HASH_CAIRO_0, URL};
use crate::helpers::fixtures::{
    copy_directory_to_tempdir, create_and_deploy_account, create_and_deploy_oz_account,
    duplicate_contract_directory_with_salt, get_accounts_path, get_transaction_hash,
    get_transaction_receipt, join_tempdirs,
};
use crate::helpers::runner::runner;
use configuration::CONFIG_FILENAME;
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};
use sncast::helpers::constants::{ARGENT_CLASS_HASH, BRAAVOS_CLASS_HASH, OZ_CLASS_HASH};
use sncast::AccountType;
use starknet::core::types::TransactionReceipt::Declare;
use starknet_types_core::felt::Felt;
use std::fs;
use test_case::test_case;

#[test_case("oz_cairo_0"; "cairo_0_account")]
#[test_case("oz_cairo_1"; "cairo_1_account")]
#[test_case("oz"; "oz_account")]
#[test_case("argent"; "argent_account")]
#[test_case("braavos"; "braavos_account")]
#[tokio::test]
async fn test_happy_case_eth(account: &str) {
    let contract_path =
        duplicate_contract_directory_with_salt(CONTRACTS_DIR.to_string() + "/map", "put", account);
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        account,
        "--int-format",
        "--json",
        "declare",
        "--url",
        URL,
        "--contract-name",
        "Map",
        "--max-fee",
        "99999999999999999",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(contract_path.path());
    let output = snapbox.assert().success().get_output().stdout.clone();

    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Declare(_)));
}

#[tokio::test]
async fn test_happy_case_human_readable() {
    let contract_path = duplicate_contract_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/map",
        "put",
        "human_readable",
    );
    let tempdir = create_and_deploy_oz_account().await;
    join_tempdirs(&contract_path, &tempdir);

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "declare",
        "--url",
        URL,
        "--contract-name",
        "Map",
        "--max-fee",
        "99999999999999999",
        "--fee-token",
        "strk",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        command: declare
        class_hash: 0x0[..]
        transaction_hash: 0x0[..]
        
        To see declaration details, visit:
        class: https://[..]
        transaction: https://[..]
    " },
    );
}

#[test_case(DEVNET_OZ_CLASS_HASH_CAIRO_0.parse().unwrap(), AccountType::OpenZeppelin; "cairo_0_class_hash")]
#[test_case(OZ_CLASS_HASH, AccountType::OpenZeppelin; "cairo_1_class_hash")]
#[test_case(ARGENT_CLASS_HASH, AccountType::Argent; "argent_class_hash")]
#[test_case(BRAAVOS_CLASS_HASH, AccountType::Braavos; "braavos_class_hash")]
#[tokio::test]
async fn test_happy_case_strk(class_hash: Felt, account_type: AccountType) {
    let contract_path = duplicate_contract_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/map",
        "put",
        &class_hash.to_string(),
    );
    let tempdir = create_and_deploy_account(class_hash, account_type).await;
    join_tempdirs(&contract_path, &tempdir);
    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "--int-format",
        "--json",
        "declare",
        "--url",
        URL,
        "--contract-name",
        "Map",
        "--max-fee",
        "99999999999999999",
        "--fee-token",
        "strk",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success().get_output().stdout.clone();

    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Declare(_)));
}

#[test_case("v2"; "v2")]
#[test_case("v3"; "v3")]
#[tokio::test]
async fn test_happy_case_versions(version: &str) {
    let contract_path =
        duplicate_contract_directory_with_salt(CONTRACTS_DIR.to_string() + "/map", "put", version);
    let tempdir = create_and_deploy_oz_account().await;
    join_tempdirs(&contract_path, &tempdir);
    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "--int-format",
        "--json",
        "declare",
        "--url",
        URL,
        "--contract-name",
        "Map",
        "--max-fee",
        "99999999999999999",
        "--version",
        version,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success().get_output().stdout.clone();

    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Declare(_)));
}

#[test_case(Some("100000000000000000"), None, None; "max_fee")]
#[test_case(None, Some("100000"), None; "max_gas")]
#[test_case(None, None, Some("100000000000000"); "max_gas_unit_price")]
#[test_case(None, None, None; "none")]
#[test_case(Some("10000000000000000000"), None, Some("100000000000000"); "max_fee_max_gas_unit_price")]
#[test_case(None, Some("100000"), Some("100000000000000"); "max_gas_max_gas_unit_price")]
#[test_case(Some("100000000000000000"), Some("100000"), None; "max_fee_max_gas")]
#[tokio::test]
async fn test_happy_case_strk_different_fees(
    max_fee: Option<&str>,
    max_gas: Option<&str>,
    max_gas_unit_price: Option<&str>,
) {
    let contract_path = duplicate_contract_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/map",
        "put",
        &format!(
            "{}{}{}",
            max_fee.unwrap_or("0"),
            max_gas.unwrap_or("1"),
            max_gas_unit_price.unwrap_or("2")
        ),
    );
    let tempdir = create_and_deploy_oz_account().await;
    join_tempdirs(&contract_path, &tempdir);
    let mut args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "--int-format",
        "--json",
        "declare",
        "--url",
        URL,
        "--contract-name",
        "Map",
        "--fee-token",
        "strk",
    ];

    let options = [
        ("--max-fee", max_fee),
        ("--max-gas", max_gas),
        ("--max-gas-unit-price", max_gas_unit_price),
    ];

    for &(key, value) in &options {
        if let Some(val) = value {
            args.append(&mut vec![key, val]);
        }
    }

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    let output = output.get_output().stdout.clone();

    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Declare(_)));
}

#[test_case("eth", "v3"; "eth-v3")]
#[test_case("strk", "v2"; "strk-v2")]
#[tokio::test]
async fn test_invalid_version_and_token_combination(fee_token: &str, version: &str) {
    let contract_path =
        duplicate_contract_directory_with_salt(CONTRACTS_DIR.to_string() + "/map", "put", version);
    let tempdir = create_and_deploy_oz_account().await;
    join_tempdirs(&contract_path, &tempdir);
    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "--int-format",
        "--json",
        "declare",
        "--url",
        URL,
        "--contract-name",
        "Map",
        "--max-fee",
        "99999999999999999",
        "--version",
        version,
        "--fee-token",
        fee_token,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        format!("Error: {fee_token} fee token is not supported for {version} declaration."),
    );
}

#[tokio::test]
async fn test_happy_case_specify_package() {
    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/multiple_packages");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user8",
        "--int-format",
        "--json",
        "declare",
        "--url",
        URL,
        "--contract-name",
        "supercomplexcode",
        "--package",
        "main_workspace",
        "--max-fee",
        "99999999999999999",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    let output = snapbox.assert().success().get_output().stdout.clone();

    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Declare(_)));
}

#[tokio::test]
async fn test_contract_already_declared() {
    let tempdir = duplicate_contract_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/map",
        "put",
        "8512851",
    );
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "declare",
        "--url",
        URL,
        "--contract-name",
        "Map",
        "--fee-token",
        "eth",
    ];

    runner(&args).current_dir(tempdir.path()).assert().success();

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: declare
        error: [..]Class with hash[..]is already declared[..]
        "},
    );
}

#[tokio::test]
async fn test_invalid_nonce() {
    let contract_path =
        duplicate_contract_directory_with_salt(CONTRACTS_DIR.to_string() + "/map", "put", "1123");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user8",
        "--int-format",
        "declare",
        "--url",
        URL,
        "--contract-name",
        "Map",
        "--max-fee",
        "99999999999999999",
        "--nonce",
        "12345",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(contract_path.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: declare
        error: Invalid transaction nonce
        "},
    );
}

#[tokio::test]
async fn test_wrong_contract_name_passed() {
    let tempdir = duplicate_contract_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/map",
        "put",
        "521754725",
    );
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "declare",
        "--url",
        URL,
        "--contract-name",
        "nonexistent",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();
    assert_stderr_contains(
        output,
        indoc! {r"
        command: declare
        error: Failed to find nonexistent artifact in starknet_artifacts.json file[..]
        "},
    );
}

#[test]
fn test_scarb_build_fails_when_wrong_cairo_path() {
    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/build_fails");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "declare",
        "--url",
        URL,
        "--contract-name",
        "BuildFails",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().failure();
    // assert_stderr_contains(
    //     output,
    //     "Failed to build contract: Failed to build using scarb; `scarb` exited with error",
    // );
    let expected = indoc! {
        "
        Error: Failed to build contract
        Caused by:
            Failed to build using scarb; `scarb` exited with error
        "
    };

    assert_stderr_contains(output, expected);
}

// #[should_panic(expected = "Path to Scarb.toml manifest does not exist")]
#[test]
fn test_scarb_build_fails_scarb_toml_does_not_exist() {
    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR);
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "declare",
        "--url",
        URL,
        "--contract-name",
        "BuildFails",
        "--fee-token",
        "eth",
    ];

    // runner(&args).current_dir(tempdir.path()).assert().success();
    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        "Error: Path to Scarb.toml manifest does not exist =[..]",
    );
}

#[test]
fn test_scarb_build_fails_manifest_does_not_exist() {
    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR);
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "declare",
        "--url",
        URL,
        "--contract-name",
        "BuildFails",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        Error: Path to Scarb.toml manifest does not exist =[..]
        "},
    );
}

#[test]
fn test_too_low_max_fee() {
    let contract_path = duplicate_contract_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/map",
        "put",
        "2451825",
    );
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user6",
        "--wait",
        "declare",
        "--url",
        URL,
        "--contract-name",
        "Map",
        "--max-fee",
        "1",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(contract_path.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: declare
        error: Max fee is smaller than the minimal transaction cost
        "},
    );
}

// #[should_panic(expected = "Make sure you have enabled sierra code generation in Scarb.toml")]
#[test]
fn test_scarb_no_sierra_artifact() {
    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/no_sierra");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "declare",
        "--url",
        URL,
        "--contract-name",
        "minimal_contract",
        "--fee-token",
        "eth",
    ];

    // runner(&args).current_dir(tempdir.path()).assert().success();
    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().failure();

    let expected = indoc! {
        "
        Error: Failed to build contract
        Caused by:
            [..]Make sure you have enabled sierra code generation in Scarb.toml[..]
        "
    };

    assert_stderr_contains(output, expected);
}

#[test]
fn test_scarb_no_casm_artifact() {
    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/no_casm");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "declare",
        "--url",
        URL,
        "--contract-name",
        "minimal_contract",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        command: declare
        class_hash: [..]
        transaction_hash: [..]
        "},
    );
}

#[tokio::test]
async fn test_many_packages_default() {
    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/multiple_packages");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user8",
        "--int-format",
        "--json",
        "declare",
        "--url",
        URL,
        "--contract-name",
        "supercomplexcode2",
        "--max-fee",
        "99999999999999999",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        "Error: More than one package found in scarb metadata - specify package using --package flag",
    );
}

#[tokio::test]
async fn test_worskpaces_package_specified_virtual_fibonacci() {
    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/virtual_workspace");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user8",
        "--int-format",
        "--json",
        "declare",
        "--url",
        URL,
        "--package",
        "cast_fibonacci",
        "--contract-name",
        "FibonacciContract",
        "--max-fee",
        "99999999999999999",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    let output = snapbox.assert().success().get_output().clone();
    let output = output.stdout.clone();
    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;
    assert!(matches!(receipt, Declare(_)));
}

#[tokio::test]
async fn test_worskpaces_package_no_contract() {
    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/virtual_workspace");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user8",
        "--int-format",
        "declare",
        "--url",
        URL,
        "--package",
        "cast_addition",
        "--contract-name",
        "whatever",
        "--max-fee",
        "99999999999999999",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: declare
        error: Failed to find whatever artifact in starknet_artifacts.json file[..]
        "},
    );
}

#[tokio::test]
async fn test_no_scarb_profile() {
    let contract_path =
        duplicate_contract_directory_with_salt(CONTRACTS_DIR.to_string() + "/map", "put", "694215");
    fs::copy(
        "tests/data/files/correct_snfoundry.toml",
        contract_path.path().join(CONFIG_FILENAME),
    )
    .expect("Failed to copy config file to temp dir");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--profile",
        "profile5",
        "declare",
        "--url",
        URL,
        "--contract-name",
        "Map",
        "--max-fee",
        "99999999999999999",
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(contract_path.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {"
            [..]
            [WARNING] Profile profile5 does not exist in scarb, using 'release' profile.
            command: declare
            class_hash: [..]
            transaction_hash: [..]

            To see declaration details, visit:
            class: [..]
            transaction: [..]
        "},
    );
}
