use crate::helpers::{constants::URL, runner::runner};
use configuration::copy_config_to_tempdir;
use indoc::formatdoc;

#[tokio::test]
async fn test_show_config_from_snfoundry_toml() {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None).unwrap();
    let args = vec!["show-config"];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(formatdoc! {r"
        command: show-config
        account: user1
        accounts_file_path: ../account-file
        chain_id: alpha-sepolia
        rpc_url: {}
        show_explorer_links: true
        wait_retry_interval: 5
        wait_timeout: 300
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
        command: show-config
        account: /path/to/account.json
        chain_id: alpha-sepolia
        keystore: ../keystore
        rpc_url: {}
        show_explorer_links: true
        wait_retry_interval: 1
        wait_timeout: 2
    ", URL});
}

#[tokio::test]
async fn test_show_config_from_cli_and_snfoundry_toml() {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None).unwrap();
    let args = vec!["--account", "user2", "--profile", "profile2", "show-config"];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(formatdoc! {r"
        command: show-config
        account: user2
        accounts_file_path: ../account-file
        chain_id: alpha-sepolia
        profile: profile2
        rpc_url: {}
        show_explorer_links: true
        wait_retry_interval: 5
        wait_timeout: 300
    ", URL});
}

#[tokio::test]
async fn test_show_config_when_no_keystore() {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None).unwrap();
    let args = vec!["--profile", "profile4", "show-config"];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(formatdoc! {r"
        command: show-config
        account: user3
        accounts_file_path: ../account-file
        chain_id: alpha-sepolia
        profile: profile4
        rpc_url: {}
        show_explorer_links: true
        wait_retry_interval: 5
        wait_timeout: 300
    ", URL});
}

#[tokio::test]
async fn test_show_config_when_keystore() {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None).unwrap();
    let args = vec!["--profile", "profile3", "show-config"];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(formatdoc! {r"
        command: show-config
        account: /path/to/account.json
        chain_id: alpha-sepolia
        keystore: ../keystore
        profile: profile3
        rpc_url: {}
        show_explorer_links: true
        wait_retry_interval: 5
        wait_timeout: 300
    ", URL});
}

#[tokio::test]
async fn test_show_config_no_url() {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None).unwrap();
    let args = vec!["--profile", "profile6", "show-config"];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(formatdoc! {r"
        command: show-config
        account: user1
        accounts_file_path: /path/to/account.json
        profile: profile6
        show_explorer_links: false
        wait_retry_interval: 10
        wait_timeout: 500
    "});
}
