use crate::helpers::{
    constants::{ACCOUNT_FILE_PATH, CONTRACTS_DIR, MAP_CONTRACT_NAME, URL},
    fixtures::{
        copy_directory_to_tempdir, duplicate_contract_directory_with_salt, get_accounts_path,
        last_line_as_json,
    },
    runner::runner,
};
use configuration::CONFIG_FILENAME;
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains, AsOutput};
use snapbox::assert_matches;
use test_case::test_case;

#[test_case("oz_cairo_0"; "cairo_0_account")]
#[test_case("oz_cairo_1"; "cairo_1_account")]
#[test_case("braavos"; "braavos_account")]
fn test_happy_case_eth(account: &str) {
    let salt = account.to_owned() + "_eth";
    let tempdir =
        duplicate_contract_directory_with_salt(CONTRACTS_DIR.to_string() + "/map", "put", &salt);

    let accounts_file = &get_accounts_path(ACCOUNT_FILE_PATH)[..];

    let args = vec![
        "--accounts-file",
        accounts_file,
        "--account",
        account,
        "--int-format",
        "--json",
        "declare-deploy",
        "--url",
        URL,
        "--contract-name",
        MAP_CONTRACT_NAME,
        "--fee-token",
        "eth",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();
    let json = last_line_as_json(output.as_stdout());

    assert_eq!(output.as_stderr(), "");
    assert_eq!(json["command"], "declare-deploy");
    assert!(json["class_hash"].as_str().is_some());
    assert!(json["contract_address"].as_str().is_some());
    assert!(json["declare_transaction_hash"].as_str().is_some());
    assert!(json["deploy_transaction_hash"].as_str().is_some());
}

#[test_case("oz"; "oz_account")]
#[test_case("argent"; "argent_account")]
fn test_happy_case_strk(account: &str) {
    let salt = account.to_owned() + "_strk";
    let tempdir =
        duplicate_contract_directory_with_salt(CONTRACTS_DIR.to_string() + "/map", "put", &salt);

    let accounts_file = &get_accounts_path(ACCOUNT_FILE_PATH)[..];

    let args = vec![
        "--accounts-file",
        accounts_file,
        "--account",
        account,
        "--int-format",
        "--json",
        "declare-deploy",
        "--url",
        URL,
        "--contract-name",
        MAP_CONTRACT_NAME,
        "--fee-token",
        "strk",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();
    let json = last_line_as_json(output.as_stdout());

    assert_eq!(output.as_stderr(), "");
    assert_eq!(json["command"], "declare-deploy");
    assert!(json["class_hash"].as_str().is_some());
    assert!(json["contract_address"].as_str().is_some());
    assert!(json["declare_transaction_hash"].as_str().is_some());
    assert!(json["deploy_transaction_hash"].as_str().is_some());
}

#[test]
fn test_happy_case_human_readable() {
    let tempdir = duplicate_contract_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/map",
        "put",
        "_human_readable",
    );

    let accounts_file = &get_accounts_path(ACCOUNT_FILE_PATH)[..];

    let args = vec![
        "--accounts-file",
        accounts_file,
        "--account",
        "user17",
        "--int-format",
        "declare-deploy",
        "--url",
        URL,
        "--contract-name",
        MAP_CONTRACT_NAME,
        "--fee-token",
        "strk",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    let expected = indoc!(
        "
        [..]
        command: [..]
        class_hash: [..]
        contract_address: [..]
        declare_transaction_hash: [..]
        deploy_transaction_hash: [..]

        To see declaration and deployment details, visit:
        class: [..]
        contract: [..]
        declaration transaction: [..]
        deployment transaction: [..]
        "
    );

    assert_stdout_contains(output, expected);
}

#[test]
#[ignore = "Expand the contract's code to more complex or wait for fix: https://github.com/xJonathanLEI/starknet-rs/issues/649#issue-2469861847"]
fn test_happy_case_specify_package() {
    let tempdir = duplicate_contract_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/multiple_packages",
        "whatever",
        "salty",
    );

    let accounts_file = &get_accounts_path(ACCOUNT_FILE_PATH)[..];

    let args = vec![
        "--accounts-file",
        accounts_file,
        "--account",
        "user18",
        "--int-format",
        "--json",
        "declare-deploy",
        "--url",
        URL,
        "--contract-name",
        "supercomplexcode",
        "--salt",
        "0x2",
        "--unique",
        "--fee-token",
        "strk",
        "--package",
        "main_workspace",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();
    let json = last_line_as_json(output.as_stdout());

    assert_eq!(output.as_stderr(), "");
    assert_eq!(json["command"], "declare-deploy");
    assert!(json["contract_address"].as_str().is_some());
    assert!(json["deploy_transaction_hash"].as_str().is_some());
}

#[test]
fn test_happy_case_contract_already_declared() {
    let tempdir = duplicate_contract_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/map",
        "put",
        "happy_case_contract_already_declared_put",
    );

    let accounts_file = &get_accounts_path("tests/data/accounts/accounts.json")[..];

    let mut args = vec![
        "--accounts-file",
        accounts_file,
        "--account",
        "user2",
        "--json",
        "declare",
        "--url",
        URL,
        "--contract-name",
        MAP_CONTRACT_NAME,
        "--fee-token",
        "strk",
    ];

    runner(&args).current_dir(tempdir.path()).assert().success();

    args[5] = "declare-deploy";

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();
    let json = last_line_as_json(output.as_stdout());

    assert_eq!(output.as_stderr(), "");
    assert_eq!(json["command"], "declare-deploy");
    assert!(json["contract_address"].as_str().is_some());
    assert!(json["deploy_transaction_hash"].as_str().is_some());
}

#[test]
fn test_nonexistent_contract() {
    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR.to_string() + "/map");
    let accounts_file = &get_accounts_path("tests/data/accounts/accounts.json")[..];

    let args = vec![
        "--accounts-file",
        accounts_file,
        "--account",
        "user3",
        "declare-deploy",
        "--url",
        URL,
        "--contract-name",
        "some_non_existent_name",
        "--fee-token",
        "strk",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        "Error: No artifacts found for contract: some_non_existent_name",
    );
}

#[test]
fn test_multiple_packages() {
    let tempdir = duplicate_contract_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/multiple_packages",
        "whatever",
        "multiple_packages",
    );

    let accounts_file = &get_accounts_path(ACCOUNT_FILE_PATH)[..];

    let args = vec![
        "--accounts-file",
        accounts_file,
        "--account",
        "user4",
        "--int-format",
        "declare-deploy",
        "--url",
        URL,
        "--contract-name",
        "supercomplexcode",
        "--salt",
        "0x2",
        "--fee-token",
        "strk",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().failure();

    let expected = "Error: More than one package found in scarb metadata - specify package using --package flag";
    assert_stderr_contains(output, expected);
}

#[test]
fn test_invalid_nonce() {
    let tempdir = duplicate_contract_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/map",
        "put",
        "salty_put",
    );

    let accounts_file = &get_accounts_path(ACCOUNT_FILE_PATH)[..];

    let args = vec![
        "--accounts-file",
        accounts_file,
        "--account",
        "user5",
        "--json",
        "declare-deploy",
        "--url",
        URL,
        "--contract-name",
        MAP_CONTRACT_NAME,
        "--fee-token",
        "strk",
        "--nonce",
        "2137",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    let output = snapbox.assert().success();
    let output = last_line_as_json(output.as_stderr());

    assert_eq!(output["command"], "declare-deploy");
    assert_matches(
        "[..]Account transaction nonce is invalid[..]",
        output["error"].as_str().unwrap(),
    );
}

#[test]
fn test_no_scarb_toml() {
    let tempdir = copy_directory_to_tempdir(CONTRACTS_DIR);
    let accounts_file = &get_accounts_path("tests/data/accounts/accounts.json")[..];

    let args = vec![
        "--accounts-file",
        accounts_file,
        "--account",
        "user6",
        "declare-deploy",
        "--url",
        URL,
        "--contract-name",
        "some_non_existent_name",
        "--fee-token",
        "strk",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        "Error: Path to Scarb.toml manifest does not exist =[..]",
    );
}

#[test]
fn test_no_scarb_profile() {
    let tempdir = duplicate_contract_directory_with_salt(
        CONTRACTS_DIR.to_string() + "/map",
        "put",
        "no_scarb_profile",
    );

    let accounts_file = &get_accounts_path(ACCOUNT_FILE_PATH)[..];

    std::fs::copy(
        "tests/data/files/correct_snfoundry.toml",
        tempdir.path().join(CONFIG_FILENAME),
    )
    .expect("Failed to copy config file to temp dir");
    let args = vec![
        "--accounts-file",
        accounts_file,
        "--account",
        "user1",
        "--profile",
        "profile5",
        "declare-deploy",
        "--url",
        URL,
        "--contract-name",
        MAP_CONTRACT_NAME,
        "--fee-token",
        "strk",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_eq!(output.as_stderr(), "");

    let expected = indoc!(
        "
        [..]
        [WARNING] Profile profile5 does not exist in scarb, using 'release' profile.
        command: [..]
        class_hash: [..]
        contract_address: [..]
        declare_transaction_hash: [..]
        deploy_transaction_hash: [..]

        To see declaration and deployment details, visit:
        class: [..]
        contract: [..]
        declaration transaction: [..]
        deployment transaction: [..]
        "
    );

    assert_stdout_contains(output, expected);
}
