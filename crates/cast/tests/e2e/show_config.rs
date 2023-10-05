use crate::helpers::runner::runner;
use indoc::indoc;

#[tokio::test]
async fn test_all_config_in_cli() {
    let args = vec![
        "--path-to-scarb-toml",
        "tests/data/show_config/all_Scarb.toml",
        "show-config",
    ];

    let snapbox = runner(&args);

    snapbox.assert().success().stdout_eq(indoc! {r#"
        command: show-config
        account: user1
        account_file_path: ../test
        chain_id: alpha-goerli
        rpc_url: http://127.0.0.1:5055/rpc
        scarb_path: tests/data/show_config/all_Scarb.toml
    "#});
}
