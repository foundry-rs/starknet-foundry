use crate::helpers::constants::{CONTRACTS_DIR, DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS, URL};
use crate::helpers::fixtures::duplicate_directory_with_salt;
use crate::helpers::runner::runner;
use camino::Utf8PathBuf;
use indoc::indoc;
use serde_json::json;
use std::fs;
use tempfile::TempDir;
use test_case::test_case;

#[tokio::test]
pub async fn test_happy_case() {
    let accounts_file = "./tmp-a1/accounts.json";
    _ = fs::remove_file(accounts_file);

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_file,
        "account",
        "add",
        "--name",
        "my_account_add",
        "--address",
        "0x123",
        "--private-key",
        "0x456",
        "--deployed",
    ];

    let snapbox = runner(&args, None);

    snapbox.assert().stdout_matches(indoc! {r"
        command: account add
        add_profile: --add-profile flag was not set. No profile added to Scarb.toml
    "});

    let contents = fs::read_to_string(accounts_file).expect("Unable to read created file");
    let contents_json: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert_eq!(
        contents_json,
        json!(
            {
                "alpha-goerli": {
                  "my_account_add": {
                    "address": "0x123",
                    "deployed": true,
                    "private_key": "0x456",
                    "public_key": "0x5f679dacd8278105bd3b84a15548fe84079068276b0e84d6cc093eb5430f063"
                  }
                }
            }
        )
    );

    fs::remove_dir_all(Utf8PathBuf::from(accounts_file).parent().unwrap()).unwrap();
}

#[test_case(false ; "Scarb.toml in current_dir")]
#[test_case(true ; "Scarb.toml passed as argument")]
#[tokio::test]
pub async fn test_happy_case_add_profile(pass_path_to_scarb_toml: bool) {
    let salt = if pass_path_to_scarb_toml { "30" } else { "31" };
    let contract_path =
        duplicate_directory_with_salt(CONTRACTS_DIR.to_string() + "/map", "put", salt);
    let contract_path_utf8 =
        Utf8PathBuf::from_path_buf(contract_path.into_path().canonicalize().unwrap().clone())
            .expect("Path contains invalid UTF-8");

    let scarb_path = contract_path_utf8.clone().join("Scarb.toml");
    let accounts_file_path = contract_path_utf8.clone().join("accounts.json");

    let mut args = vec![];
    if pass_path_to_scarb_toml {
        args.append(&mut vec!["--path-to-scarb-toml", scarb_path.as_str()]);
    }

    args.append(&mut vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_file_path.as_str(),
        "account",
        "add",
        "--name",
        "my_account_add",
        "--address",
        "0x1",
        "--private-key",
        "0x2",
        "--public-key",
        "0x759ca09377679ecd535a81e83039658bf40959283187c654c5416f439403cf5",
        "--salt",
        "0x3",
        "--class-hash",
        "0x4",
        "--add-profile",
    ]);
    let current_dir = if pass_path_to_scarb_toml {
        None
    } else {
        Some(contract_path_utf8.as_std_path())
    };
    let snapbox = runner(&args, current_dir);

    snapbox.assert().stdout_matches(indoc! {r"
        command: account add
        add_profile: Profile successfully added to Scarb.toml
    "});

    let contents = fs::read_to_string(accounts_file_path).expect("Unable to read created file");
    let contents_json: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert_eq!(
        contents_json,
        json!(
            {
                "alpha-goerli": {
                  "my_account_add": {
                    "address": "0x1",
                    "class_hash": "0x4",
                    "deployed": false,
                    "private_key": "0x2",
                    "public_key": "0x759ca09377679ecd535a81e83039658bf40959283187c654c5416f439403cf5",
                    "salt": "0x3",
                  }
                }
            }
        )
    );

    let contents = fs::read_to_string(scarb_path).expect("Unable to read Scarb.toml");
    assert!(contents.contains("[tool.sncast.my_account_add]"));
    assert!(contents.contains("account = \"my_account_add\""));
}

#[tokio::test]
pub async fn test_detect_deployed() {
    let accounts_file = "./tmp-a2/accounts.json";
    _ = fs::remove_file(accounts_file);

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        accounts_file,
        "account",
        "add",
        "--name",
        "my_account_add",
        "--address",
        DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS,
        "--private-key",
        "0x5",
    ];

    let snapbox = runner(&args, None);

    snapbox.assert().stdout_matches(indoc! {r"
        command: account add
        add_profile: --add-profile flag was not set. No profile added to Scarb.toml
    "});

    let contents = fs::read_to_string(accounts_file).expect("Unable to read created file");
    let contents_json: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert_eq!(
        contents_json,
        json!(
            {
                "alpha-goerli": {
                  "my_account_add": {
                    "address": DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS,
                    "deployed": true,
                    "private_key": "0x5",
                    "public_key": "0x788435d61046d3eec54d77d25bd194525f4fa26ebe6575536bc6f656656b74c"
                  }
                }
            }
        )
    );

    fs::remove_dir_all(Utf8PathBuf::from(accounts_file).parent().unwrap()).unwrap();
}

#[tokio::test]
pub async fn test_invalid_public_key() {
    let args = vec![
        "--url",
        URL,
        "account",
        "add",
        "--name",
        "my_account_add",
        "--address",
        "0x123",
        "--private-key",
        "0x456",
        "--public-key",
        "0x457",
        "--deployed",
    ];

    let snapbox = runner(&args, None);

    snapbox.assert().stderr_matches(indoc! {r"
        command: account add
        error: The private key does not match the public key
    "});
}

#[tokio::test]
pub async fn test_missing_arguments() {
    let args = vec![
        "--url",
        URL,
        "account",
        "add",
        "--name",
        "my_account_add",
        "--deployed",
    ];

    let snapbox = runner(&args, None);
    snapbox.assert().stderr_matches(indoc! {r"
        error: the following required arguments were not provided:
          --address <ADDRESS>
          <--private-key <PRIVATE_KEY>|--private-key-file <PRIVATE_KEY_FILE_PATH>>
        ...
    "});
}

#[tokio::test]
pub async fn test_private_key_from_file() {
    let temp_dir = TempDir::new().expect("Unable to create a temporary directory");
    let accounts_file = "./accounts.json";
    let private_key_file = "./my_private_key";

    fs::write(temp_dir.path().join(private_key_file), "0x456").unwrap();

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        "./accounts.json",
        "account",
        "add",
        "--name",
        "my_account_add",
        "--address",
        "0x123",
        "--private-key-file",
        private_key_file,
        "--deployed",
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(temp_dir.path())
        .args(args);

    snapbox.assert().stdout_matches(indoc! {r"
        command: account add
        add_profile: --add-profile flag was not set. No profile added to Scarb.toml
    "});

    let contents = fs::read_to_string(temp_dir.path().join(accounts_file))
        .expect("Unable to read created file");
    let contents_json: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert_eq!(
        contents_json,
        json!(
            {
                "alpha-goerli": {
                  "my_account_add": {
                    "address": "0x123",
                    "deployed": true,
                    "private_key": "0x456",
                    "public_key": "0x5f679dacd8278105bd3b84a15548fe84079068276b0e84d6cc093eb5430f063"
                  }
                }
            }
        )
    );
}

#[tokio::test]
pub async fn test_accept_only_one_private_key() {
    let args = vec![
        "account",
        "add",
        "--name",
        "my_account_add",
        "--address",
        "0x123",
        "--private-key",
        "0x456",
        "--private-key-file",
        "./my_private_key",
    ];

    let snapbox = runner(&args);
    snapbox.assert().stderr_matches(indoc! {r"
        error: the argument '--private-key <PRIVATE_KEY>' cannot be used with '--private-key-file <PRIVATE_KEY_FILE_PATH>'
        ...
    "});
}

#[tokio::test]
pub async fn test_invalid_private_key_file_path() {
    let args = vec![
        "--url",
        URL,
        "account",
        "add",
        "--name",
        "my_account_add",
        "--address",
        "0x123",
        "--private-key-file",
        "./my_private_key",
        "--deployed",
    ];

    let snapbox = runner(&args);

    snapbox.assert().stderr_matches(indoc! {r"
        command: account add
        error: Failed to obtain private key from the file [..]
    "});
}
