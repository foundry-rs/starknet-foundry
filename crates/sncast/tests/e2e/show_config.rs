use crate::helpers::runner::runner;
use configuration::copy_config_to_tempdir;
use indoc::indoc;

#[tokio::test]
async fn test_show_config_from_snfoundry_toml() {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None);
    let args = vec!["show-config"];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(indoc! {r"
        command: show-config
        account: user1
        accounts_file_path: ../account-file
        chain_id: alpha-goerli
        rpc_url: http://127.0.0.1:5055/rpc
        wait_retry_interval: 5
        wait_timeout: 300
    "});
}

#[tokio::test]
async fn test_show_config_from_cli() {
    let args = vec![
        "--account",
        "/path/to/account.json",
        "--url",
        "http://127.0.0.1:5055/rpc",
        "--keystore",
        "../keystore",
        "--wait-timeout",
        "2",
        "--wait-retry-interval",
        "1",
        "show-config",
    ];

    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r"
        command: show-config
        account: /path/to/account.json
        chain_id: alpha-goerli
        keystore: ../keystore
        rpc_url: http://127.0.0.1:5055/rpc
        wait_retry_interval: 1
        wait_timeout: 2
    "});
}

#[tokio::test]
async fn test_show_config_from_cli_and_snfoundry_toml() {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None);
    let args = vec!["--account", "user2", "--profile", "profile2", "show-config"];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(indoc! {r"
        command: show-config
        account: user2
        accounts_file_path: ../account-file
        chain_id: alpha-goerli
        profile: profile2
        rpc_url: http://127.0.0.1:5055/rpc
        wait_retry_interval: 5
        wait_timeout: 300
    "});
}

#[tokio::test]
async fn test_show_config_when_no_keystore() {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None);
    let args = vec!["--profile", "profile4", "show-config"];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(indoc! {r"
        command: show-config
        account: user3
        accounts_file_path: ../account-file
        chain_id: alpha-goerli
        profile: profile4
        rpc_url: http://127.0.0.1:5055/rpc
        wait_retry_interval: 5
        wait_timeout: 300
    "});
}

#[tokio::test]
async fn test_show_config_when_keystore() {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None);
    let args = vec!["--profile", "profile3", "show-config"];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(indoc! {r"
        command: show-config
        account: /path/to/account.json
        chain_id: alpha-goerli
        keystore: ../keystore
        profile: profile3
        rpc_url: http://127.0.0.1:5055/rpc
        wait_retry_interval: 5
        wait_timeout: 300
    "});
}
