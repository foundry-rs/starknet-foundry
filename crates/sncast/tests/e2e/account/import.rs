use crate::helpers::constants::{
    DEVNET_OZ_CLASS_HASH_CAIRO_0, DEVNET_OZ_CLASS_HASH_CAIRO_1, DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS,
    URL,
};
use crate::helpers::runner::runner;
use camino::Utf8PathBuf;
use configuration::CONFIG_FILENAME;
use conversions::string::IntoHexStr;
use indoc::{formatdoc, indoc};
use serde_json::json;
use shared::test_utils::output_assert::assert_stderr_contains;
use std::fs::{self, File};
use tempfile::tempdir;
use test_case::test_case;

#[test_case("oz", "open_zeppelin"; "oz_account_type")]
#[test_case("argent", "argent"; "argent_account_type")]
#[test_case("braavos", "braavos"; "braavos_account_type")]
#[tokio::test]
pub async fn test_happy_case(input_account_type: &str, saved_type: &str) {
    let tempdir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "import",
        "--url",
        URL,
        "--name",
        "my_account_import",
        "--address",
        "0x123",
        "--private-key",
        "0x456",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH_CAIRO_0,
        "--type",
        input_account_type,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().stdout_matches(indoc! {r"
        command: account import
        add_profile: --add-profile flag was not set. No profile added to snfoundry.toml
    "});

    let contents = fs::read_to_string(tempdir.path().join(accounts_file))
        .expect("Unable to read created file");
    let contents_json: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert_eq!(
        contents_json,
        json!(
            {
                "alpha-sepolia": {
                  "my_account_import": {
                    "address": "0x123",
                    "class_hash": DEVNET_OZ_CLASS_HASH_CAIRO_0,
                    "deployed": false,
                    "legacy": true,
                    "private_key": "0x456",
                    "public_key": "0x5f679dacd8278105bd3b84a15548fe84079068276b0e84d6cc093eb5430f063",
                    "type": saved_type
                  }
                }
            }
        )
    );
}

#[tokio::test]
pub async fn test_existent_account_address() {
    let tempdir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "import",
        "--url",
        URL,
        "--name",
        "my_account_import",
        "--address",
        DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS,
        "--private-key",
        "0x456",
        "--type",
        "oz",
    ];

    runner(&args).current_dir(tempdir.path()).assert();

    let contents = fs::read_to_string(tempdir.path().join(accounts_file))
        .expect("Unable to read created file");
    let contents_json: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert_eq!(
        contents_json,
        json!(
            {
                "alpha-sepolia": {
                  "my_account_import": {
                    "address": DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS,
                    "class_hash": &DEVNET_OZ_CLASS_HASH_CAIRO_1.into_hex_string(),
                    "deployed": true,
                    "legacy": false,
                    "private_key": "0x456",
                    "public_key": "0x5f679dacd8278105bd3b84a15548fe84079068276b0e84d6cc093eb5430f063",
                    "type": "open_zeppelin"
                  }
                }
            }
        )
    );
}

#[tokio::test]
pub async fn test_existent_account_address_and_incorrect_class_hash() {
    let tempdir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "import",
        "--url",
        URL,
        "--name",
        "my_account_import",
        "--address",
        DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS,
        "--private-key",
        "0x456",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH_CAIRO_0,
        "--type",
        "oz",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().stderr_matches(formatdoc! {r"
        command: account import
        error: Incorrect class hash {} for account address {} was provided
    ", DEVNET_OZ_CLASS_HASH_CAIRO_0, DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS});
}

#[tokio::test]
pub async fn test_nonexistent_account_address_and_nonexistent_class_hash() {
    let tempdir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "import",
        "--url",
        URL,
        "--name",
        "my_account_import",
        "--address",
        "0x202",
        "--private-key",
        "0x456",
        "--class-hash",
        "0x101",
        "--type",
        "oz",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().stderr_matches(indoc! {r"
        command: account import
        error: Class with hash 0x101 is not declared, try using --class-hash with a hash of the declared class
    "});
}

#[tokio::test]
pub async fn test_nonexistent_account_address() {
    let tempdir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "import",
        "--url",
        URL,
        "--name",
        "my_account_import",
        "--address",
        "0x123",
        "--private-key",
        "0x456",
        "--type",
        "oz",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().stderr_matches(indoc! {r"
        command: account import
        error: Class hash for the account address 0x123 could not be found. Please provide the class hash
    "});
}

#[tokio::test]
pub async fn test_happy_case_add_profile() {
    let tempdir = tempdir().expect("Failed to create a temporary directory");
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "import",
        "--url",
        URL,
        "--name",
        "my_account_import",
        "--address",
        "0x1",
        "--private-key",
        "0x2",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH_CAIRO_0,
        "--type",
        "oz",
        "--add-profile",
        "my_account_import",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().stdout_matches(indoc! {r"
        command: account import
        add_profile: Profile my_account_import successfully added to snfoundry.toml
    "});
    let current_dir_utf8 = Utf8PathBuf::try_from(tempdir.path().to_path_buf()).unwrap();

    let contents = fs::read_to_string(current_dir_utf8.join(accounts_file))
        .expect("Unable to read created file");
    let contents_json: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert_eq!(
        contents_json,
        json!(
            {
                "alpha-sepolia": {
                  "my_account_import": {
                    "address": "0x1",
                    "class_hash": DEVNET_OZ_CLASS_HASH_CAIRO_0,
                    "deployed": false,
                    "private_key": "0x2",
                    "public_key": "0x759ca09377679ecd535a81e83039658bf40959283187c654c5416f439403cf5",
                    "legacy": true,
                    "type": "open_zeppelin"
                  }
                }
            }
        )
    );

    let contents = fs::read_to_string(current_dir_utf8.join("snfoundry.toml"))
        .expect("Unable to read snfoundry.toml");
    assert!(contents.contains("[sncast.my_account_import]"));
    assert!(contents.contains("account = \"my_account_import\""));
    assert!(contents.contains(&format!("url = \"{URL}\"")));
}

#[tokio::test]
pub async fn test_detect_deployed() {
    let tempdir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "import",
        "--url",
        URL,
        "--name",
        "my_account_import",
        "--address",
        DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS,
        "--private-key",
        "0x5",
        "--type",
        "oz",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().stdout_matches(indoc! {r"
        command: account import
        add_profile: --add-profile flag was not set. No profile added to snfoundry.toml
    "});

    let contents = fs::read_to_string(tempdir.path().join(accounts_file))
        .expect("Unable to read created file");
    let contents_json: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert_eq!(
        contents_json,
        json!(
            {
                "alpha-sepolia": {
                  "my_account_import": {
                    "address": DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS,
                    "class_hash": &DEVNET_OZ_CLASS_HASH_CAIRO_1.into_hex_string(),
                    "deployed": true,
                    "private_key": "0x5",
                    "public_key": "0x788435d61046d3eec54d77d25bd194525f4fa26ebe6575536bc6f656656b74c",
                    "legacy": false,
                    "type": "open_zeppelin"
                  }
                }
            }
        )
    );
}

#[tokio::test]
pub async fn test_missing_arguments() {
    let args = vec![
        "account",
        "import",
        "--url",
        URL,
        "--name",
        "my_account_import",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        error: the following required arguments were not provided:
          --address <ADDRESS>
          --type <ACCOUNT_TYPE>
        "},
    );
}

#[tokio::test]
pub async fn test_private_key_from_file() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts.json";
    let private_key_file = "my_private_key";

    fs::write(temp_dir.path().join(private_key_file), "0x456").unwrap();

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "import",
        "--url",
        URL,
        "--name",
        "my_account_import",
        "--address",
        "0x123",
        "--private-key-file",
        private_key_file,
        "--class-hash",
        DEVNET_OZ_CLASS_HASH_CAIRO_0,
        "--type",
        "oz",
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());

    snapbox.assert().stdout_matches(indoc! {r"
        command: account import
        add_profile: --add-profile flag was not set. No profile added to snfoundry.toml
    "});

    let contents = fs::read_to_string(temp_dir.path().join(accounts_file))
        .expect("Unable to read created file");
    let contents_json: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert_eq!(
        contents_json,
        json!(
            {
                "alpha-sepolia": {
                  "my_account_import": {
                    "address": "0x123",
                    "deployed": false,
                    "legacy": true,
                    "private_key": "0x456",
                    "public_key": "0x5f679dacd8278105bd3b84a15548fe84079068276b0e84d6cc093eb5430f063",
                    "class_hash": DEVNET_OZ_CLASS_HASH_CAIRO_0,
                    "type": "open_zeppelin"
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
        "import",
        "--name",
        "my_account_import",
        "--address",
        "0x123",
        "--private-key",
        "0x456",
        "--private-key-file",
        "my_private_key",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        "error: the argument '--private-key <PRIVATE_KEY>' cannot be used with '--private-key-file <PRIVATE_KEY_FILE_PATH>'"
    );
}

#[tokio::test]
pub async fn test_invalid_private_key_file_path() {
    let args = vec![
        "account",
        "import",
        "--url",
        URL,
        "--name",
        "my_account_import",
        "--address",
        "0x123",
        "--private-key-file",
        "my_private_key",
        "--type",
        "oz",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: account import
        error: Failed to obtain private key from the file my_private_key: No such file or directory (os error 2)
        "},
    );
}

#[tokio::test]
pub async fn test_invalid_private_key_in_file() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let private_key_file = "my_private_key";

    fs::write(
        temp_dir.path().join(private_key_file),
        "invalid private key",
    )
    .unwrap();

    let args = vec![
        "--accounts-file",
        "accounts.json",
        "account",
        "import",
        "--url",
        URL,
        "--name",
        "my_account_import",
        "--address",
        "0x123",
        "--private-key-file",
        private_key_file,
        "--type",
        "oz",
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        command: account import
        error: Failed to obtain private key from the file my_private_key: Failed to create Felt from string
        "},
    );
}

#[tokio::test]
pub async fn test_private_key_as_int_in_file() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts.json";
    let private_key_file = "my_private_key";

    fs::write(temp_dir.path().join(private_key_file), "1110").unwrap();

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "import",
        "--url",
        URL,
        "--name",
        "my_account_import",
        "--address",
        DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS,
        "--private-key-file",
        private_key_file,
        "--type",
        "oz",
    ];

    runner(&args)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    let contents = fs::read_to_string(temp_dir.path().join(accounts_file))
        .expect("Unable to read created file");
    let contents_json: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert_eq!(
        contents_json,
        json!(
            {
                "alpha-sepolia": {
                  "my_account_import": {
                    "address": DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS,
                    "deployed": true,
                    "legacy": false,
                    "private_key": "0x456",
                    "public_key": "0x5f679dacd8278105bd3b84a15548fe84079068276b0e84d6cc093eb5430f063",
                    "class_hash": &DEVNET_OZ_CLASS_HASH_CAIRO_1.into_hex_string(),
                    "type": "open_zeppelin"
                  }
                }
            }
        )
    );
}

#[tokio::test]
pub async fn test_empty_config_add_profile() {
    let tempdir = tempdir().expect("Unable to create a temporary directory");
    File::create(tempdir.path().join(CONFIG_FILENAME)).unwrap();
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "import",
        "--url",
        URL,
        "--name",
        "my_account_import",
        "--address",
        DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS,
        "--private-key",
        "0x456",
        "--type",
        "oz",
        "--add-profile",
        "random",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().stdout_matches(indoc! {r"
        command: account import
        add_profile: Profile random successfully added to snfoundry.toml
    "});
    let current_dir_utf8 = Utf8PathBuf::try_from(tempdir.path().to_path_buf()).unwrap();

    let contents = fs::read_to_string(current_dir_utf8.join("snfoundry.toml"))
        .expect("Unable to read snfoundry.toml");
    assert!(contents.contains("[sncast.random]"));
    assert!(contents.contains("account = \"my_account_import\""));
    assert!(contents.contains(&format!("url = \"{URL}\"")));
}

#[tokio::test]
pub async fn test_happy_case_valid_address_computation() {
    let tempdir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "import",
        "--url",
        URL,
        "--name",
        "my_account_import",
        "--address",
        "0x721c21e0cc9d37aec8e176797effd1be222aff6db25f068040adefabb7cfb6d",
        "--private-key",
        "0x2",
        "--salt",
        "0x3",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH_CAIRO_0,
        "--type",
        "oz",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().stdout_matches(indoc! {r"
        command: account import
        add_profile: --add-profile flag was not set. No profile added to snfoundry.toml
    "});

    let contents = fs::read_to_string(tempdir.path().join(accounts_file))
        .expect("Unable to read created file");
    let contents_json: serde_json::Value = serde_json::from_str(&contents).unwrap();
    assert_eq!(
        contents_json,
        json!(
            {
                "alpha-sepolia": {
                  "my_account_import": {
                    "address": "0x721c21e0cc9d37aec8e176797effd1be222aff6db25f068040adefabb7cfb6d",
                    "class_hash": DEVNET_OZ_CLASS_HASH_CAIRO_0,
                    "deployed": false,
                    "salt": "0x3",
                    "legacy": true,
                    "private_key": "0x2",
                    "public_key": "0x759ca09377679ecd535a81e83039658bf40959283187c654c5416f439403cf5",
                    "type": "open_zeppelin"
                  }
                }
            }
        )
    );
}

#[tokio::test]
pub async fn test_invalid_address_computation() {
    let tempdir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "import",
        "--url",
        URL,
        "--name",
        "my_account_import",
        "--address",
        "0x123",
        "--private-key",
        "0x456",
        "--salt",
        "0x789",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH_CAIRO_0,
        "--type",
        "oz",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let computed_address = "0xaf550326d32c8106ef08d25cbc0dba06e5cd10a2201c2e9bc5ad4cbbce45e6";
    snapbox.assert().stderr_matches(formatdoc! {r"
        command: account import
        error: Computed address {computed_address} does not match the provided address 0x123. Please ensure that the provided salt, class hash, and account type are correct.
    "});
}
