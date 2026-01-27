use crate::e2e::ledger::{TEST_LEDGER_PATH, automation, setup_speculos};
use crate::helpers::runner::runner;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_get_app_version() {
    let (_client, url) = setup_speculos(4001);

    let output = runner(&["ledger", "app-version"])
        .env("LEDGER_EMULATOR_URL", &url)
        .assert()
        .success();

    assert_stdout_contains(output, "App Version: 2.3.4");
}

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_get_public_key_headless() {
    let (_client, url) = setup_speculos(4002);

    let output = runner(&[
        "ledger",
        "get-public-key",
        "--path",
        TEST_LEDGER_PATH,
        "--no-display",
    ])
    .env("LEDGER_EMULATOR_URL", &url)
    .assert()
    .success();

    assert_stdout_contains(
        output,
        "Public Key: 0x0051f3e99d539868d8f45ca705ad6f75e68229a6037a919b15216b4e92a4d6d8",
    );
}

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_get_public_key_with_confirmation() {
    let (client, url) = setup_speculos(4003);

    client
        .automation(&[automation::APPROVE_PUBLIC_KEY])
        .await
        .unwrap();

    let output = runner(&["ledger", "get-public-key", "--path", TEST_LEDGER_PATH])
        .env("LEDGER_EMULATOR_URL", &url)
        .assert()
        .success();

    assert_stdout_contains(
        output,
        "Public Key: 0x0051f3e99d539868d8f45ca705ad6f75e68229a6037a919b15216b4e92a4d6d8",
    );
}

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_sign_hash() {
    let (client, url) = setup_speculos(4004);

    client
        .automation(&[
            automation::ENABLE_BLIND_SIGN,
            automation::APPROVE_BLIND_SIGN_HASH,
        ])
        .await
        .unwrap();

    let output = runner(&[
        "ledger",
        "sign-hash",
        "--path",
        TEST_LEDGER_PATH,
        "0x01234567890abcdef1234567890abcdef1234567890abcdef1234567890abcd",
    ])
    .env("LEDGER_EMULATOR_URL", &url)
    .assert()
    .success();

    assert_stdout_contains(output, "Signature: 0x[..]");
}

#[tokio::test]
#[ignore = "requires Speculos installation"]
async fn test_sign_hash_invalid_format() {
    let (_client, url) = setup_speculos(4005);

    let output = runner(&[
        "ledger",
        "sign-hash",
        "--path",
        TEST_LEDGER_PATH,
        "not-a-hex-hash",
    ])
    .env("LEDGER_EMULATOR_URL", &url)
    .assert()
    .success();

    assert_stderr_contains(output, "Error: Failed to parse hash[..]");
}
