use crate::helpers::constants::{CONTRACTS_DIR, URL};
use crate::helpers::fixtures::{
    copy_directory_to_tempdir, duplicate_contract_directory_with_salt, get_accounts_path,
    get_transaction_hash, get_transaction_receipt,
};
use indoc::indoc;
use snapbox::cmd::{cargo_bin, Command};
use sncast::helpers::constants::CONFIG_FILENAME;
use starknet::core::types::TransactionReceipt::Declare;
use std::fs;

#[tokio::test]
async fn test_happy_case() {
    let contract_path =
        duplicate_contract_directory_with_salt(CONTRACTS_DIR.to_string() + "/map", "put", "1");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user8",
        "--int-format",
        "--json",
        "declare",
        "--contract-name",
        "Map",
        "--max-fee",
        "99999999999999999",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(contract_path.path())
        .args(args);
    let output = snapbox.assert().success().get_output().stdout.clone();

    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Declare(_)));
}

#[tokio::test]
async fn test_happy_case_specify_package() {
    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/multiple_packages");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user8",
        "--int-format",
        "--json",
        "declare",
        "--contract-name",
        "supercomplexcode",
        "--package",
        "main_workspace",
        "--max-fee",
        "99999999999999999",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(tempdir.path())
        .args(args);
    let output = snapbox.assert().success().get_output().stdout.clone();

    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Declare(_)));
}

#[tokio::test]
async fn contract_already_declared() {
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        "../../accounts/accounts.json",
        "--account",
        "user1",
        "declare",
        "--contract-name",
        "Map",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(CONTRACTS_DIR.to_string() + "/map")
        .args(args);

    snapbox.assert().success().stderr_matches(indoc! {r"
        command: declare
        error: An error occurred in the called contract [..]
    "});
}

#[tokio::test]
async fn wrong_contract_name_passed() {
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        "../../accounts/accounts.json",
        "--account",
        "user1",
        "declare",
        "--contract-name",
        "nonexistent",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(CONTRACTS_DIR.to_string() + "/map")
        .args(args);

    snapbox.assert().success().stderr_matches(indoc! {r"
        command: declare
        error: Failed to find nonexistent artifact in starknet_artifacts.json file[..]
    "});
}

#[test]
fn scarb_build_fails_when_wrong_cairo_path() {
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        "../../accounts/accounts.json",
        "--account",
        "user1",
        "declare",
        "--contract-name",
        "BuildFails",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(CONTRACTS_DIR.to_string() + "/build_fails")
        .args(args);

    snapbox.assert().stderr_matches(indoc! {r"
        ...
        Failed to build contract: Failed to build using scarb; `scarb` exited with error
        ...
    "});
}

#[should_panic(expected = "Path to Scarb.toml manifest does not exist")]
#[test]
fn scarb_build_fails_scarb_toml_does_not_exist() {
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        "../accounts/accounts.json",
        "--account",
        "user1",
        "declare",
        "--contract-name",
        "BuildFails",
    ];

    Command::new(cargo_bin!("sncast"))
        .current_dir(CONTRACTS_DIR.to_string() + "/")
        .args(args)
        .assert()
        .success();
}

#[test]
fn scarb_build_fails_manifest_does_not_exist() {
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        "../accounts/accounts.json",
        "--account",
        "user1",
        "declare",
        "--contract-name",
        "BuildFails",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(CONTRACTS_DIR.to_string() + "/")
        .args(args);

    snapbox.assert().stderr_matches(indoc! {r"
        Error: Path to Scarb.toml manifest does not exist =[..]
    "});
}

#[test]
fn test_too_low_max_fee() {
    let contract_path =
        duplicate_contract_directory_with_salt(CONTRACTS_DIR.to_string() + "/map", "put", "2");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user6",
        "--wait",
        "declare",
        "--contract-name",
        "Map",
        "--max-fee",
        "1",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(contract_path.path())
        .args(args);

    snapbox.assert().success().stderr_matches(indoc! {r"
        command: declare
        error: Max fee is smaller than the minimal transaction cost
    "});
}

#[should_panic(expected = "Make sure you have enabled sierra code generation in Scarb.toml")]
#[test]
fn scarb_no_sierra_artifact() {
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        "../../accounts/accounts.json",
        "--account",
        "user1",
        "declare",
        "--contract-name",
        "minimal_contract",
    ];

    Command::new(cargo_bin!("sncast"))
        .current_dir(CONTRACTS_DIR.to_string() + "/no_sierra")
        .args(args)
        .assert()
        .success();
}

#[test]
fn scarb_no_casm_artifact() {
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        "../../accounts/accounts.json",
        "--account",
        "user1",
        "declare",
        "--contract-name",
        "minimal_contract",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(CONTRACTS_DIR.to_string() + "/no_casm")
        .args(args);

    assert!(
        String::from_utf8(snapbox.assert().success().get_output().stdout.clone())
            .unwrap()
            .contains("class_hash")
    );
}

#[tokio::test]
async fn test_many_packages_default() {
    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/multiple_packages");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user8",
        "--int-format",
        "--json",
        "declare",
        "--contract-name",
        "supercomplexcode2",
        "--max-fee",
        "99999999999999999",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(tempdir.path())
        .args(args);
    snapbox.assert().failure().stderr_matches(indoc! {r"
        ...
        Error: More than one package found in scarb metadata - specify package using --package flag
    "});
}

#[tokio::test]
async fn test_worskpaces_package_specified_virtual_fibonacci() {
    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/virtual_workspace");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user8",
        "--int-format",
        "--json",
        "declare",
        "--package",
        "cast_fibonacci",
        "--contract-name",
        "FibonacciContract",
        "--max-fee",
        "99999999999999999",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(tempdir.path())
        .args(args);

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
        "--url",
        URL,
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user8",
        "--int-format",
        "declare",
        "--package",
        "cast_addition",
        "--contract-name",
        "whatever",
        "--max-fee",
        "99999999999999999",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(tempdir.path())
        .args(args);
    snapbox.assert().success().stderr_matches(indoc! {r"
        ...
        command: declare
        error: Failed to find whatever artifact in starknet_artifacts.json file[..]
    "});
}

#[tokio::test]
async fn test_no_scarb_profile() {
    let contract_path =
        duplicate_contract_directory_with_salt(CONTRACTS_DIR.to_string() + "/map", "put", "69");
    fs::copy(
        "tests/data/files/correct_snfoundry.toml",
        contract_path.path().join(CONFIG_FILENAME),
    )
    .expect("Failed to copy config file to temp dir");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");
    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_json_path.as_str(),
        "--profile",
        "profile5",
        "declare",
        "--contract-name",
        "Map",
        "--max-fee",
        "99999999999999999",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(contract_path.path())
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        Warning: Profile profile5 does not exist in scarb, using default 'dev' profile.
        command: declare
        class_hash: [..]
        transaction_hash: [..]
    "});
}
