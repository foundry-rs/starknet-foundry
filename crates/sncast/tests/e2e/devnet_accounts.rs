use crate::helpers::constants::{MAP_CONTRACT_ADDRESS_SEPOLIA, URL};
use crate::helpers::fixtures::copy_file;
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};
use tempfile::tempdir;
use test_case::test_case;

#[test_case(1)]
#[test_case(20)]
#[tokio::test]
pub async fn test_happy_case(account_number: u8) {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");

    let account = format!("devnet-{account_number}");
    let args = vec![
        "--account",
        &account,
        "invoke",
        "--url",
        URL,
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "put",
        "--calldata",
        "0x1 0x2",
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {
            "
            Success: Invoke completed

            Transaction Hash: 0x0[..]
            "
        },
    );
}

#[test_case(0)]
#[test_case(21)]
#[tokio::test]
pub async fn test_account_number_out_of_range(account_number: u8) {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");

    let account = format!("devnet-{account_number}");
    let args = vec![
        "--account",
        &account,
        "invoke",
        "--url",
        URL,
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "put",
        "--calldata",
        "0x1 0x2",
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {
            "
            Error: Devnet account number must be between 1 and 20
            "
        },
    );
}

#[tokio::test]
pub async fn test_account_name_already_exists() {
    let accounts_file = "accounts.json";
    let temp_dir = tempdir().expect("Unable to create a temporary directory");

    copy_file(
        "tests/data/accounts/accounts.json",
        temp_dir.path().join(accounts_file),
    );

    let args = vec![
        "--accounts-file",
        accounts_file,
        "--account",
        "devnet-1",
        "invoke",
        "--url",
        URL,
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "put",
        "--calldata",
        "0x1 0x2",
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {
            "
            [WARNING] Using account devnet-1 from accounts file accounts.json. To use inbuilt devnet account, please change the name of your existing account devnet-1.
            
            Success: Invoke completed

            Transaction Hash: 0x0[..]
            "
        },
    );
}
