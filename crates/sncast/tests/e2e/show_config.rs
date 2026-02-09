use crate::helpers::{constants::URL, runner::runner};
use configuration::test_utils::copy_config_to_tempdir;
use indoc::formatdoc;
use shared::test_utils::output_assert::assert_stderr_contains;

#[tokio::test]
async fn test_show_config_from_snfoundry_toml() {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None);
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
    ", URL});
}

#[tokio::test]
async fn test_show_config_from_cli_and_snfoundry_toml() {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None);
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
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None);
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
    ", URL});
}

#[tokio::test]
async fn test_show_config_when_keystore() {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None);
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
    ", URL});
}

#[tokio::test]
async fn test_show_config_no_url() {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None);
    let args = vec!["--profile", "profile6", "show-config"];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(formatdoc! {r"
        Profile:             profile6
        Account:             user1
        Accounts File Path:  /path/to/account.json
        Wait Timeout:        500s
        Wait Retry Interval: 10s
        Show Explorer Links: false
    "});
}

#[tokio::test]
async fn test_show_config_with_network() {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None);
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
    "});
}

#[tokio::test]
async fn test_only_one_from_url_and_network_allowed() {
    let tempdir = copy_config_to_tempdir("tests/data/files/invalid_snfoundry.toml", None);
    let args = vec!["--profile", "url_and_network", "show-config"];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        "Error: Failed to load config: Only one of `url` or `network` may be specified",
    );
}

#[tokio::test]
async fn test_stark_scan_as_block_explorer() {
    let tempdir = copy_config_to_tempdir("tests/data/files/invalid_snfoundry.toml", None);
    let args = vec!["--profile", "profile_with_stark_scan", "show-config"];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        "Error: Failed to load config: starkscan.co was terminated and `'StarkScan'` is no longer available. Please set `block-explorer` to `'Voyager'` or other explorer of your choice.",
    );
}
