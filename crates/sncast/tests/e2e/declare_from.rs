use crate::helpers::constants::{MAP_CONTRACT_CLASS_HASH_SEPOLIA, SEPOLIA_RPC_URL, URL};
use crate::helpers::fee::apply_test_resource_bounds_flags;
use crate::helpers::fixtures::get_accounts_path;
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};
use tempfile::tempdir;

#[tokio::test]
async fn test_happy_case() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let example_contract_class_hash_sepolia =
        "0x283a4f96ee7de15894d9205a93db7cec648562cfe90db14cb018c039e895e78";

    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "declare-from",
        "--class-hash",
        example_contract_class_hash_sepolia,
        "--source-url",
        SEPOLIA_RPC_URL,
        "--url",
        URL,
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        Success: Declaration completed

        Class Hash:       0x[..]
        Transaction Hash: 0x[..]
        
        To see declaration details, visit:
        class: https://[..]
        transaction: https://[..]
    " },
    );
}

#[tokio::test]
async fn test_happy_case_with_block_id() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let example_b_contract_class_hash_sepolia =
        "0x3de1a95e27b385c882c79355ca415915989e71f67c0f6f8ce146d4bcee7163c";

    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user2",
        "declare-from",
        "--class-hash",
        example_b_contract_class_hash_sepolia,
        "--source-url",
        SEPOLIA_RPC_URL,
        "--url",
        URL,
        "--block-id",
        "latest",
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        Success: Declaration completed

        Class Hash:       0x[..]
        Transaction Hash: 0x[..]
        
        To see declaration details, visit:
        class: https://[..]
        transaction: https://[..]
    " },
    );
}

#[tokio::test]
async fn test_contract_already_declared() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user3",
        "declare-from",
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--source-url",
        SEPOLIA_RPC_URL,
        "--url",
        URL,
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: declare-from
        Error: Contract with the same class hash is already declared
        "},
    );
}

#[tokio::test]
async fn test_class_hash_does_not_exist_on_source_network() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "declare-from",
        "--class-hash",
        "0x1",
        "--source-url",
        SEPOLIA_RPC_URL,
        "--url",
        URL,
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: declare-from
        Error: Provided class hash does not exist
        "},
    );
}

#[tokio::test]
async fn test_source_rpc_args_not_passed() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "declare-from",
        "--class-hash",
        "0x1",
        "--url",
        URL,
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        Error: Either `--source-network` or `--source-url` must be provided
        "},
    );
}

#[tokio::test]
async fn test_invalid_block_id() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_json_path = get_accounts_path("tests/data/accounts/accounts.json");

    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user1",
        "declare-from",
        "--class-hash",
        "0x1",
        "--url",
        URL,
        "--block-id",
        "0x10101",
    ];
    let args = apply_test_resource_bounds_flags(args);

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        Error: Either `--source-network` or `--source-url` must be provided
        "},
    );
}
