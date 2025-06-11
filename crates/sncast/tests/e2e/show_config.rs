use crate::helpers::{constants::URL, runner::runner};
use configuration::copy_config_to_tempdir;
use indoc::formatdoc;

#[tokio::test]
async fn test_show_config_from_snfoundry_toml() {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None).unwrap();
    let args = vec!["show-config"];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(formatdoc! {r"
        Success: Configuration details

        Chain ID:            alpha-sepolia
        RPC URL:             {}
        Account:             user1
        Accounts File Path:  ../account-file
        Wait Timeout:        300
        Wait Retry Interval: 5
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
        Success: Configuration details

        Chain ID:            alpha-sepolia
        RPC URL:             {}
        Account:             /path/to/account.json
        Keystore:            ../keystore
        Wait Timeout:        2
        Wait Retry Interval: 1
        Show Explorer Links: true
    ", URL});
}

#[tokio::test]
async fn test_show_config_from_cli_and_snfoundry_toml() {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None).unwrap();
    let args = vec!["--account", "user2", "--profile", "profile2", "show-config"];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(formatdoc! {r"
        Success: Configuration details

        Profile:             profile2
        Chain ID:            alpha-sepolia
        RPC URL:             {}
        Account:             user2
        Accounts File Path:  ../account-file
        Wait Timeout:        300
        Wait Retry Interval: 5
        Show Explorer Links: true
    ", URL});
}

#[tokio::test]
async fn test_show_config_when_no_keystore() {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None).unwrap();
    let args = vec!["--profile", "profile4", "show-config"];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(formatdoc! {r"
        Success: Configuration details

        Profile:             profile4
        Chain ID:            alpha-sepolia
        RPC URL:             {}
        Account:             user3
        Accounts File Path:  ../account-file
        Wait Timeout:        300
        Wait Retry Interval: 5
        Show Explorer Links: true
    ", URL});
}

#[tokio::test]
async fn test_show_config_when_keystore() {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None).unwrap();
    let args = vec!["--profile", "profile3", "show-config"];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(formatdoc! {r"
        Success: Configuration details

        Profile:             profile3
        Chain ID:            alpha-sepolia
        RPC URL:             {}
        Account:             /path/to/account.json
        Keystore:            ../keystore
        Wait Timeout:        300
        Wait Retry Interval: 5
        Show Explorer Links: true
    ", URL});
}

#[tokio::test]
async fn test_show_config_no_url() {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None).unwrap();
    let args = vec!["--profile", "profile6", "show-config"];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(formatdoc! {r"
        Success: Configuration details
        
        Profile:             profile6
        Account:             user1
        Accounts File Path:  /path/to/account.json
        Wait Timeout:        500
        Wait Retry Interval: 10
        Show Explorer Links: false
    "});
}
