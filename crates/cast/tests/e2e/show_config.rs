use crate::helpers::runner::runner;
use indoc::indoc;

#[tokio::test]
async fn test_show_config_from_scarb_toml() {
    let args = vec![
        "--path-to-scarb-toml",
        "tests/data/show_config/all_Scarb.toml",
        "--profile",
        "profile1",
        "show-config",
    ];

    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r#"
        command: show-config
        account: user1
        accounts_file_path: ../account-file
        chain_id: alpha-goerli
        profile: profile1
        rpc_url: http://127.0.0.1:5055/rpc
        scarb_path: tests/data/show_config/all_Scarb.toml
    "#});
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
        "show-config",
    ];

    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r#"
        command: show-config
        account: /path/to/account.json
        chain_id: alpha-goerli
        keystore: ../keystore
        rpc_url: http://127.0.0.1:5055/rpc
    "#});
}

#[tokio::test]
async fn test_show_config_from_cli_and_scarb() {
    let args = vec![
        "--account",
        "user2",
        "--path-to-scarb-toml",
        "tests/data/show_config/all_Scarb.toml",
        "--profile",
        "profile1",
        "show-config",
    ];

    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r#"
        command: show-config
        account: user2
        accounts_file_path: ../account-file
        chain_id: alpha-goerli
        profile: profile1
        rpc_url: http://127.0.0.1:5055/rpc
        scarb_path: tests/data/show_config/all_Scarb.toml
    "#});
}

#[tokio::test]
async fn test_show_config_when_no_keystore() {
    let args = vec![
        "--path-to-scarb-toml",
        "tests/data/show_config/all_Scarb.toml",
        "--profile",
        "profile1",
        "show-config",
    ];

    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r#"
        command: show-config
        account: user1
        accounts_file_path: ../account-file
        chain_id: alpha-goerli
        profile: profile1
        rpc_url: http://127.0.0.1:5055/rpc
        scarb_path: tests/data/show_config/all_Scarb.toml
    "#});
}

#[tokio::test]
async fn test_show_config_when_keystore() {
    let args = vec![
        "--path-to-scarb-toml",
        "tests/data/show_config/all_Scarb.toml",
        "show-config",
    ];

    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r#"
        command: show-config
        account: /path/to/account.json
        chain_id: alpha-goerli
        keystore: ../keystore
        rpc_url: http://127.0.0.1:5055/rpc
        scarb_path: tests/data/show_config/all_Scarb.toml
    "#});
}
