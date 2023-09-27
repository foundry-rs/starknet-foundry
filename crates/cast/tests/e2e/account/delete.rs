use crate::helpers::constants::DEVNET_OZ_CLASS_HASH;
use crate::helpers::fixtures::default_cli_args;
use crate::helpers::runner::runner;
use indoc::indoc;

#[tokio::test]
pub async fn test_account_network_should_exist() {
    let mut args = default_cli_args();
    args.append(&mut vec![
        "account",
        "delete",
        "--name",
        "user99",
        "--network",
        "goerli0-network",
    ]);

    let snapbox = runner(&args);

    snapbox.assert().stderr_matches(indoc! {r#"
    command: account delete
    error: No accounts defined for network goerli0-network
    "#});
}

#[tokio::test]
pub async fn test_account_delete_should_exist() {
    let mut args = default_cli_args();
    args.append(&mut vec!["account", "delete", "--name", "user99"]);

    let snapbox = runner(&args);

    snapbox.assert().stderr_matches(indoc! {r#"
    command: account delete
    error: Account with name user99 does not exist
    "#});
}

#[tokio::test]
pub async fn test_account_delete_happy_case() {
    // First, create a dummy account
    let mut args = default_cli_args();
    args.append(&mut vec![
        "account",
        "create",
        "--name",
        "user99",
        "--salt",
        "0x1",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ]);

    let snapbox = runner(&args);

    snapbox.assert();

    // Now delete dummy account
    args = default_cli_args();
    args.append(&mut vec!["account", "delete", "--name", "user99"]);

    let snapbox1 = runner(&args);

    let bdg = snapbox1.assert();
    let out = bdg.get_output();

    let stdout_str =
        std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");
    assert!(stdout_str.contains("Account successfully removed"));
}
