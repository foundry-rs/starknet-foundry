use crate::helpers::constants::{MAP_CONTRACT_ADDRESS_SEPOLIA, SEPOLIA_RPC_URL, URL};
use crate::helpers::fixtures::copy_file;
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};
use tempfile::tempdir;
use test_case::test_case;

#[test_case(1)]
#[test_case(20)]
#[tokio::test]
pub async fn happy_case(account_number: u8) {
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
pub async fn account_number_out_of_range(account_number: u8) {
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
pub async fn account_name_already_exists() {
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
            [WARNING] Using account devnet-1 from accounts file accounts.json. To use an inbuilt devnet account, please rename your existing account or use an account with a different number.
            
            Success: Invoke completed

            Transaction Hash: 0x0[..]
            "
        },
    );
}

#[tokio::test]
pub async fn use_devnet_account_with_network_not_being_devnet() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");

    let args = vec![
        "--account",
        "devnet-1",
        "invoke",
        "--url",
        SEPOLIA_RPC_URL,
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
        format! {"Error: Node at {SEPOLIA_RPC_URL} is not responding to the Devnet health check (GET `/is_alive`). It may not be a Devnet instance or it may be down."
        },
    );
}

#[test_case("mainnet")]
#[test_case("sepolia")]
#[tokio::test]
pub async fn use_devnet_account_with_network_flags(network: &str) {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");

    let args = vec![
        "--account",
        "devnet-1",
        "invoke",
        "--network",
        network,
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
        format! {"Error: Devnet accounts cannot be used with `--network {network}`"
        },
    );
}
