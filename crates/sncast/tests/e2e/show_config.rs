use crate::helpers::{
    constants::URL,
    runner::{Cast, runner},
};
use configuration::test_utils::copy_config_to_tempdir;
use indoc::{formatdoc, indoc};
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};
use std::fs;
use tempfile::tempdir;

#[tokio::test]
async fn test_show_config_from_snfoundry_toml() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_correct.toml", None);
    let args = vec!["show-config"];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(formatdoc! {r"
        Chain ID:            alpha-sepolia
        RPC URL:             {}
        Account:             user1
        Accounts File Path:  ../account-file
        Wait Timeout:        300s
        Wait Retry Interval: 5s
        Show Explorer Links: true
        Block Explorer:      Voyager
    ", URL});
}

#[tokio::test]
async fn test_show_config_from_cli() {
    let args = vec![
        "--account",
        "/path/to/account.json",
        "--keystore",
        "../keystore",
        "--wait-timeout",
        "2",
        "--wait-retry-interval",
        "1",
        "show-config",
        "--url",
        URL,
    ];

    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(formatdoc! {r"
        Chain ID:            alpha-sepolia
        RPC URL:             {}
        Account:             /path/to/account.json
        Keystore:            ../keystore
        Wait Timeout:        2s
        Wait Retry Interval: 1s
        Show Explorer Links: true
        Block Explorer:      Voyager
    ", URL});
}

#[tokio::test]
async fn test_show_config_from_cli_and_snfoundry_toml() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_correct.toml", None);
    let args = vec!["--account", "user2", "--profile", "profile2", "show-config"];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(formatdoc! {r"
        Profile:             profile2
        Chain ID:            alpha-sepolia
        RPC URL:             {}
        Account:             user2
        Accounts File Path:  ../account-file
        Wait Timeout:        300s
        Wait Retry Interval: 5s
        Show Explorer Links: true
        Block Explorer:      ViewBlock
    ", URL});
}

#[tokio::test]
async fn test_show_config_when_no_keystore() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_correct.toml", None);
    let args = vec!["--profile", "profile4", "show-config"];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(formatdoc! {r"
        Profile:             profile4
        Chain ID:            alpha-sepolia
        RPC URL:             {}
        Account:             user3
        Accounts File Path:  ../account-file
        Wait Timeout:        300s
        Wait Retry Interval: 5s
        Show Explorer Links: true
        Block Explorer:      Voyager
    ", URL});
}

#[tokio::test]
async fn test_show_config_when_keystore() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_correct.toml", None);
    let args = vec!["--profile", "profile3", "show-config"];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(formatdoc! {r"
        Profile:             profile3
        Chain ID:            alpha-sepolia
        RPC URL:             {}
        Account:             /path/to/account.json
        Keystore:            ../keystore
        Wait Timeout:        300s
        Wait Retry Interval: 5s
        Show Explorer Links: true
        Block Explorer:      Voyager
    ", URL});
}

#[tokio::test]
async fn test_show_config_no_url() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_correct.toml", None);
    let args = vec!["--profile", "profile6", "show-config"];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(formatdoc! {r"
        Profile:             profile6
        Chain ID:            alpha-sepolia
        RPC URL:             {}
        Account:             user1
        Accounts File Path:  /path/to/account.json
        Wait Timeout:        500s
        Wait Retry Interval: 10s
        Show Explorer Links: false
        Block Explorer:      Voyager
    ", URL});
}

#[tokio::test]
async fn test_show_config_with_network() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_correct.toml", None);
    let args = vec!["--profile", "profile7", "show-config"];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(formatdoc! {r"
        Profile:             profile7
        Chain ID:            alpha-sepolia
        Network:             sepolia
        Account:             user1
        Accounts File Path:  /path/to/account.json
        Wait Timeout:        300s
        Wait Retry Interval: 5s
        Show Explorer Links: true
        Block Explorer:      Voyager
    "});
}

#[tokio::test]
async fn test_show_config_cli_url_overrides_config_network() {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None);
    let args = vec![
        "--profile",
        "profile7",
        "show-config",
        "--url",
        "http://127.0.0.1:1111",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    snapbox.assert().success().stdout_eq(formatdoc! {r"
        Profile:             profile7
        RPC URL:             http://127.0.0.1:1111/
        Account:             user1
        Accounts File Path:  /path/to/account.json
        Wait Timeout:        300s
        Wait Retry Interval: 5s
        Show Explorer Links: true
        Block Explorer:      Voyager
    "});
}

#[tokio::test]
async fn test_only_one_from_url_and_network_allowed() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_invalid.toml", None);
    let args = vec!["--profile", "url_and_network", "show-config"];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! { r"
            Error: Failed to load local config at [..]snfoundry.toml

            Caused by:
                Only one of `url` or `network` may be specified
        " },
    );
}

#[tokio::test]
async fn test_stark_scan_as_block_explorer() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_invalid.toml", None);
    let args = vec!["--profile", "profile_with_stark_scan", "show-config"];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! { r"
            Error: Failed to load local config at [..]snfoundry.toml

            Caused by:
                starkscan.co was terminated and `'StarkScan'` is no longer available. Please set `block-explorer` to `'Voyager'` or other explorer of your choice.
        " },
    );
}

#[tokio::test]
async fn test_show_config_malformed() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_malformed.toml", None);
    let args = vec!["show-config"];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! { r"
            Error: Failed to load local config at [..]snfoundry.toml

            Caused by:
                0: Failed to parse snfoundry.toml config file
                1: TOML parse error at line 2, column 10
        " },
    );
}

#[tokio::test]
async fn test_show_config_provider_error() {
    let t = tempdir().unwrap();
    fs::write(
        t.path().join("snfoundry.toml"),
        indoc! {r#"
            [sncast.default]
            url = "http://127.0.0.1:1/rpc"
            account = "user1"
        "#},
    )
    .unwrap();
    let args = vec!["show-config"];

    let snapbox = runner(&args).current_dir(t.path());

    assert_stdout_contains(
        snapbox.assert().success(),
        indoc! {r"
            Could not reach RPC provider: Error while calling RPC method spec_version: error sending request for url (http://127.0.0.1:1/rpc)
            RPC URL:             http://127.0.0.1:1/rpc
            Account:             user1
            Accounts File Path:  [..]/.starknet_accounts/starknet_open_zeppelin_accounts.json
            Wait Timeout:        300s
            Wait Retry Interval: 5s
            Show Explorer Links: true
            Block Explorer:      Voyager
        "},
    );
}

#[tokio::test]
async fn test_show_config_global_no_local() {
    let global_dir = copy_config_to_tempdir("tests/data/files/snfoundry_global_correct.toml", None);
    let t = tempdir().unwrap();
    let args = vec!["show-config"];

    let snapbox = Cast::new()
        .config_dir(global_dir.path())
        .command()
        .args(&args)
        .current_dir(t.path());

    snapbox.assert().success().stdout_eq(formatdoc! {r"
        Chain ID:            alpha-sepolia
        RPC URL:             {}
        Account:             global_default_user
        Accounts File Path:  ../global-account-file
        Wait Timeout:        120s
        Wait Retry Interval: 3s
        Show Explorer Links: true
        Block Explorer:      Voyager
    ", URL});
}

#[tokio::test]
async fn test_show_config_global_only_profile() {
    let global_dir = copy_config_to_tempdir("tests/data/files/snfoundry_global_correct.toml", None);
    let t = tempdir().unwrap();
    let args = vec!["--profile", "global_only_profile", "show-config"];

    let snapbox = Cast::new()
        .config_dir(global_dir.path())
        .command()
        .args(&args)
        .current_dir(t.path());

    snapbox.assert().success().stdout_eq(formatdoc! {r"
        Profile:             global_only_profile
        Chain ID:            alpha-sepolia
        RPC URL:             {}
        Account:             global_profile_user
        Accounts File Path:  ../global-account-file
        Wait Timeout:        123s
        Wait Retry Interval: 5s
        Show Explorer Links: false
        Block Explorer:      Voyager
    ", URL});
}

#[tokio::test]
async fn test_show_config_global_and_local_default() {
    let global_dir = copy_config_to_tempdir("tests/data/files/snfoundry_global_correct.toml", None);
    let local_dir = copy_config_to_tempdir("tests/data/files/snfoundry_correct.toml", None);
    let args = vec!["show-config"];

    let snapbox = Cast::new()
        .config_dir(global_dir.path())
        .command()
        .args(&args)
        .current_dir(local_dir.path());

    snapbox.assert().success().stdout_eq(formatdoc! {r"
        Chain ID:            alpha-sepolia
        RPC URL:             {}
        Account:             user1
        Accounts File Path:  ../account-file
        Wait Timeout:        120s
        Wait Retry Interval: 3s
        Show Explorer Links: true
        Block Explorer:      Voyager
    ", URL});
}

#[tokio::test]
async fn test_show_config_global_and_local_profile() {
    let global_dir = copy_config_to_tempdir("tests/data/files/snfoundry_global_correct.toml", None);
    let local_dir = copy_config_to_tempdir("tests/data/files/snfoundry_correct.toml", None);
    let args = vec!["--profile", "profile2", "show-config"];

    let snapbox = Cast::new()
        .config_dir(global_dir.path())
        .command()
        .args(&args)
        .current_dir(local_dir.path());

    snapbox.assert().success().stdout_eq(formatdoc! {r"
        Profile:             profile2
        Chain ID:            alpha-sepolia
        RPC URL:             {}
        Account:             user100
        Accounts File Path:  ../account-file
        Wait Timeout:        120s
        Wait Retry Interval: 3s
        Show Explorer Links: true
        Block Explorer:      ViewBlock
    ", URL});
}

#[tokio::test]
async fn test_default_global_profile_with_invalid_values() {
    let global_dir =
        copy_config_to_tempdir("tests/data/files/snfoundry_invalid_default.toml", None);
    let t = tempdir().unwrap();
    let args = vec!["show-config"];

    let snapbox = Cast::new()
        .config_dir(global_dir.path())
        .command()
        .args(&args)
        .current_dir(t.path());

    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! { r"
            Error: Failed to load global config at [..]snfoundry.toml

            Caused by:
                starkscan.co was terminated and `'StarkScan'` is no longer available. Please set `block-explorer` to `'Voyager'` or other explorer of your choice.
        " },
    );
}

#[tokio::test]
async fn test_default_global_profile_with_unsupported_field() {
    let global_dir = copy_config_to_tempdir(
        "tests/data/files/snfoundry_invalid_unknown_field.toml",
        None,
    );
    let t = tempdir().unwrap();
    let args = vec!["show-config"];

    let snapbox = Cast::new()
        .config_dir(global_dir.path())
        .command()
        .args(&args)
        .current_dir(t.path());

    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! { r#"
            Error: Failed to load global config at [..]snfoundry.toml

            Caused by:
                unknown field(s) ["bar", "baz", "foo"]
        "# },
    );
}
