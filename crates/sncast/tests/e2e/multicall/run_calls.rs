use crate::helpers::constants::{
    MAP_CONTRACT_ADDRESS_SEPOLIA, MAP_CONTRACT_CLASS_HASH_SEPOLIA, URL,
};
use crate::helpers::fixtures::create_and_deploy_account;
use crate::helpers::runner::runner;
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};
use sncast::AccountType;
use sncast::helpers::constants::OZ_CLASS_HASH;

#[tokio::test]
async fn test_two_invokes() {
    let tempdir = create_and_deploy_account(OZ_CLASS_HASH, AccountType::OpenZeppelin).await;

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "multicall",
        "--url",
        URL,
        "invoke",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--calldata",
        "0x1 0x2",
        "--function",
        "put",
        "/",
        "invoke",
        "--contract-address",
        MAP_CONTRACT_ADDRESS_SEPOLIA,
        "--function",
        "put",
        "--calldata",
        "0x3 0x4",
    ];

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {
            "
            Success: Multicall completed

            Transaction Hash: 0x[..]

            To see invocation details, visit:
            transaction: [..]
            "
        },
    );
}

#[tokio::test]
async fn test_deploy_and_invoke() {
    let tempdir = create_and_deploy_account(OZ_CLASS_HASH, AccountType::OpenZeppelin).await;

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "multicall",
        "--url",
        URL,
        "deploy",
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "/",
        "invoke",
        "--contract-address",
        "0x00cd8f9ab31324bb93251837e4efb4223ee195454f6304fcfcb277e277653008",
        "--function",
        "get",
        "--calldata",
        "0x1",
    ];

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {
            "
            Success: Multicall completed

            Transaction Hash: 0x[..]

            To see invocation details, visit:
            transaction: [..]
            "
        },
    );
}

#[tokio::test]
async fn test_use_id() {
    let tempdir = create_and_deploy_account(OZ_CLASS_HASH, AccountType::OpenZeppelin).await;

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "multicall",
        "--url",
        URL,
        "deploy",
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--id",
        "dpl",
        "/",
        "invoke",
        "--contract-address",
        "@dpl",
        "--function",
        "get",
        "--calldata",
        "@dpl",
    ];

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {
            "
            Success: Multicall completed

            Transaction Hash: 0x[..]

            To see invocation details, visit:
            transaction: [..]
            "
        },
    );
}

#[tokio::test]
async fn test_non_existent_id() {
    let tempdir = create_and_deploy_account(OZ_CLASS_HASH, AccountType::OpenZeppelin).await;

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "multicall",
        "--url",
        URL,
        "deploy",
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--id",
        "dpl",
        "/",
        "invoke",
        "--contract-address",
        "@non_existent_id",
        "--function",
        "get",
        "--calldata",
        "0x1",
    ];

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {
            "
            Error: No contract address found for id: non_existent_id. Ensure the referenced id is defined in a previous step.
            "
        },
    );
}

#[tokio::test]
async fn test_duplicated_id() {
    let tempdir = create_and_deploy_account(OZ_CLASS_HASH, AccountType::OpenZeppelin).await;

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "--account",
        "my_account",
        "multicall",
        "--url",
        URL,
        "deploy",
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--id",
        "dpl",
        "/",
        "deploy",
        "--class-hash",
        MAP_CONTRACT_CLASS_HASH_SEPOLIA,
        "--id",
        "dpl",
    ];

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {
            "
            Error: Duplicate id found: dpl
            "
        },
    );
}
