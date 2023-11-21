use crate::helpers::runner::runner;
use indoc::indoc;
use std::path::Path;
use test_case::test_case;

#[test_case(Some(Path::new("tests/data/show_config")), None ; "Scarb.toml in current_dir")]
#[test_case(None, Some("tests/data/show_config/Scarb.toml") ; "Scarb.toml passed as argument")]
#[tokio::test]
async fn test_show_config_from_scarb_toml(
    current_dir: Option<&Path>,
    path_to_scarb_toml: Option<&str>,
) {
    let mut args = vec![];
    if let Some(scarb_path) = path_to_scarb_toml {
        args.append(&mut vec!["--path-to-scarb-toml", scarb_path]);
    }
    args.append(&mut vec!["--profile", "profile1", "show-config"]);

    let snapbox = runner(&args, current_dir);
    let mut expected_output = String::from(indoc! {r#"
        command: show-config
        account: user1
        accounts_file_path: ../account-file
        chain_id: alpha-goerli
        profile: profile1
        rpc_url: http://127.0.0.1:5055/rpc
    "#});
    if let Some(scarb_path) = path_to_scarb_toml {
        expected_output.push_str(&format!("scarb_path: {scarb_path}\n"));
    }
    snapbox.assert().success().stdout_eq(expected_output);
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

    let snapbox = runner(&args, None);

    snapbox.assert().success().stdout_eq(indoc! {r#"
        command: show-config
        account: /path/to/account.json
        chain_id: alpha-goerli
        keystore: ../keystore
        rpc_url: http://127.0.0.1:5055/rpc
    "#});
}

#[test_case(Some(Path::new("tests/data/show_config")), None ; "Scarb.toml in current_dir")]
#[test_case(None, Some("tests/data/show_config/Scarb.toml") ; "Scarb.toml passed as argument")]
#[tokio::test]
async fn test_show_config_from_cli_and_scarb(
    current_dir: Option<&Path>,
    path_to_scarb_toml: Option<&str>,
) {
    let mut args = vec!["--account", "user2"];
    if let Some(scarb_path) = path_to_scarb_toml {
        args.append(&mut vec!["--path-to-scarb-toml", scarb_path]);
    }
    args.append(&mut vec!["--profile", "profile1", "show-config"]);
    let snapbox = runner(&args, current_dir);

    let mut expected_output = String::from(indoc! {r#"
        command: show-config
        account: user2
        accounts_file_path: ../account-file
        chain_id: alpha-goerli
        profile: profile1
        rpc_url: http://127.0.0.1:5055/rpc
    "#});
    if let Some(scarb_path) = path_to_scarb_toml {
        expected_output.push_str(&format!("scarb_path: {scarb_path}\n"));
    }
    snapbox.assert().success().stdout_eq(expected_output);
}

#[test_case(Some(Path::new("tests/data/show_config")), None ; "Scarb.toml in current_dir")]
#[test_case(None, Some("tests/data/show_config/Scarb.toml") ; "Scarb.toml passed as argument")]
#[tokio::test]
async fn test_show_config_when_no_keystore(
    current_dir: Option<&Path>,
    path_to_scarb_toml: Option<&str>,
) {
    let mut args = vec![];
    if let Some(scarb_path) = path_to_scarb_toml {
        args.append(&mut vec!["--path-to-scarb-toml", scarb_path]);
    }
    args.append(&mut vec!["--profile", "profile1", "show-config"]);

    let snapbox = runner(&args, current_dir);
    let mut expected_output = String::from(indoc! {r#"
        command: show-config
        account: user1
        accounts_file_path: ../account-file
        chain_id: alpha-goerli
        profile: profile1
        rpc_url: http://127.0.0.1:5055/rpc
    "#});
    if let Some(scarb_path) = path_to_scarb_toml {
        expected_output.push_str(&format!("scarb_path: {scarb_path}\n"));
    }
    snapbox.assert().success().stdout_eq(expected_output);
}

#[test_case(Some(Path::new("tests/data/show_config")), None ; "Scarb.toml in current_dir")]
#[test_case(None, Some("tests/data/show_config/Scarb.toml") ; "Scarb.toml passed as argument")]
#[tokio::test]
async fn test_show_config_when_keystore(
    current_dir: Option<&Path>,
    path_to_scarb_toml: Option<&str>,
) {
    let mut args = vec![];
    if let Some(scarb_path) = path_to_scarb_toml {
        args.append(&mut vec!["--path-to-scarb-toml", scarb_path]);
    }
    args.append(&mut vec!["show-config"]);

    let snapbox = runner(&args, current_dir);

    let mut expected_output = String::from(indoc! {r#"
        command: show-config
        account: /path/to/account.json
        chain_id: alpha-goerli
        keystore: ../keystore
        rpc_url: http://127.0.0.1:5055/rpc
    "#});
    if let Some(scarb_path) = path_to_scarb_toml {
        expected_output.push_str(&format!("scarb_path: {scarb_path}\n"));
    }
    snapbox.assert().success().stdout_eq(expected_output);
}
