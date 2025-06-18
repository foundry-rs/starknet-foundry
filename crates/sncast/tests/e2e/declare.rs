use crate::helpers::constants::{CONTRACTS_DIR, DEVNET_OZ_CLASS_HASH_CAIRO_0, URL};
use crate::helpers::fee::apply_test_resource_bounds_flags;
use crate::helpers::fixtures::{
    copy_directory_to_tempdir, create_and_deploy_account, create_and_deploy_oz_account,
    duplicate_contract_directory_with_salt, get_accounts_path, get_transaction_hash,
    get_transaction_receipt, join_tempdirs,
};
use crate::helpers::runner::runner;
use configuration::CONFIG_FILENAME;
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};
use sncast::AccountType;
use sncast::helpers::constants::{ARGENT_CLASS_HASH, BRAAVOS_CLASS_HASH, OZ_CLASS_HASH};
use sncast::helpers::fee::FeeArgs;
use starknet::core::types::TransactionReceipt::Declare;
use starknet_types_core::felt::{Felt, NonZeroFelt};
use std::fs;
use test_case::test_case;

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
    ];
    let args = apply_test_resource_bounds_flags(args);

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

#[test_case(DEVNET_OZ_CLASS_HASH_CAIRO_0.parse().unwrap(), AccountType::OpenZeppelin; "cairo_0_class_hash"
)]
#[test_case(OZ_CLASS_HASH, AccountType::OpenZeppelin; "cairo_1_class_hash")]
#[test_case(ARGENT_CLASS_HASH, AccountType::Argent; "argent_class_hash")]
#[test_case(BRAAVOS_CLASS_HASH, AccountType::Braavos; "braavos_class_hash")]
#[tokio::test]
async fn test_happy_case(class_hash: Felt, account_type: AccountType) {
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
        "--json",
        "declare",
        "--url",
        URL,
        "--contract-name",
        "Map",
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success().get_output().stdout.clone();

    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Declare(_)));
}

#[test_case(FeeArgs{
    max_fee: Some(NonZeroFelt::try_from(Felt::from(1_000_000_000_000_000_000_000_000_u128)).unwrap()),
    l1_data_gas: None,
    l1_data_gas_price:  None,
    l1_gas:  None,
    l1_gas_price:  None,
    l2_gas:  None,
    l2_gas_price:  None,
}; "max_fee")]
#[test_case(FeeArgs{
    max_fee: None,
    l1_data_gas: Some(100_000),
    l1_data_gas_price: Some(10_000_000_000_000),
    l1_gas: Some(100_000),
    l1_gas_price: Some(10_000_000_000_000),
    l2_gas: Some(1_000_000_000),
    l2_gas_price: Some(100_000_000_000_000_000_000),
}; "resource_bounds")]
#[tokio::test]
async fn test_happy_case_different_fees(fee_args: FeeArgs) {
    let contract_path = duplicate_contract_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/map",
        "put",
        &format!(
            "{}{}{}{}{}{}{}",
            fee_args.max_fee.map_or(Felt::from(0), Felt::from),
            fee_args.l1_data_gas.unwrap_or(1),
            fee_args.l1_data_gas_price.unwrap_or(2),
            fee_args.l1_gas.unwrap_or(3),
            fee_args.l1_gas_price.unwrap_or(4),
            fee_args.l2_gas.unwrap_or(5),
            fee_args.l2_gas_price.unwrap_or(6)
        ),
    );
    let tempdir = create_and_deploy_oz_account().await;
    join_tempdirs(&contract_path, &tempdir);
    let mut args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "--json",
        "declare",
        "--url",
        URL,
        "--contract-name",
        "Map",
    ];

    let options = [
        (
            "--max-fee",
            fee_args.max_fee.map(Felt::from).map(|x| x.to_string()),
        ),
        ("--l1-data-gas", fee_args.l1_data_gas.map(|x| x.to_string())),
        (
            "--l1-data-gas-price",
            fee_args.l1_data_gas_price.map(|x| x.to_string()),
        ),
        ("--l1-gas", fee_args.l1_gas.map(|x| x.to_string())),
        (
            "--l1-gas-price",
            fee_args.l1_gas_price.map(|x| x.to_string()),
        ),
        ("--l2-gas", fee_args.l2_gas.map(|x| x.to_string())),
        (
            "--l2-gas-price",
            fee_args.l2_gas_price.map(|x| x.to_string()),
        ),
    ];

    for &(key, ref value) in &options {
        if let Some(val) = value.as_ref() {
            args.push(key);
            args.push(val);
        }
    }

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    let output = output.get_output().stdout.clone();

    let hash = get_transaction_hash(&output);
    let receipt = get_transaction_receipt(hash).await;

    assert!(matches!(receipt, Declare(_)));
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
        "--json",
        "declare",
        "--url",
        URL,
        "--contract-name",
        "supercomplexcode",
        "--package",
        "main_workspace",
    ];
    let args = apply_test_resource_bounds_flags(args);

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
    ];
    let args = apply_test_resource_bounds_flags(args);

    runner(&args).current_dir(tempdir.path()).assert().success();

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: declare
        error: Contract with the same class hash is already declared
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
        "declare",
        "--url",
        URL,
        "--contract-name",
        "Map",
        "--nonce",
        "12345",
    ];
    let args = apply_test_resource_bounds_flags(args);

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
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().failure();
    assert_stderr_contains(
        output,
        "Failed to build contract: Failed to build using scarb; `scarb` exited with error",
    );
}

#[should_panic(expected = "Path to Scarb.toml manifest does not exist")]
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
    ];

    runner(&args).current_dir(tempdir.path()).assert().success();
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
fn test_too_low_gas() {
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
        "--l1-gas",
        "1",
        "--l2-gas",
        "1",
        "--l1-data-gas",
        "1",
    ];

    let snapbox = runner(&args).current_dir(contract_path.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: declare
        error: The transaction's resources don't cover validation or the minimal transaction fee
        "},
    );
}

#[should_panic(expected = "Make sure you have enabled sierra code generation in Scarb.toml")]
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
    ];

    runner(&args).current_dir(tempdir.path()).assert().success();
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
    ];
    let args = apply_test_resource_bounds_flags(args);

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
        "--json",
        "declare",
        "--url",
        URL,
        "--contract-name",
        "supercomplexcode2",
    ];
    let args = apply_test_resource_bounds_flags(args);

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
        "--json",
        "declare",
        "--url",
        URL,
        "--package",
        "cast_fibonacci",
        "--contract-name",
        "FibonacciContract",
    ];
    let args = apply_test_resource_bounds_flags(args);

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
        "declare",
        "--url",
        URL,
        "--package",
        "cast_addition",
        "--contract-name",
        "whatever",
    ];
    let args = apply_test_resource_bounds_flags(args);

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
    ];
    let args = apply_test_resource_bounds_flags(args);

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
