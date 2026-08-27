use crate::helpers::constants::{
    DEVNET_OZ_CLASS_HASH_CAIRO_0, DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS, URL,
};
use crate::helpers::insta::set_snapshot_suffix;
use crate::helpers::runner::runner;
use camino::Utf8PathBuf;
use configuration::CONFIG_FILENAME;
use configuration::test_utils::copy_config_to_tempdir;
use indoc::{formatdoc, indoc};
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};
use std::fs::{self, File};
use tempfile::tempdir;
use test_case::test_case;

#[test_case("oz"; "open_zeppelin")]
#[test_case("ready"; "ready")]
#[test_case("braavos"; "braavos")]
#[tokio::test]
pub async fn test_happy_case(input_account_type: &str) {
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

    snapbox.assert().stdout_eq(indoc! {r"
        Success: Account imported successfully

        Account Name: my_account_import
    "});

    let contents = fs::read_to_string(tempdir.path().join(accounts_file))
        .expect("Unable to read created file");

    set_snapshot_suffix!("{input_account_type}");
    insta::assert_snapshot!(contents);
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

    let _ = runner(&args).current_dir(tempdir.path()).assert();

    let contents = fs::read_to_string(tempdir.path().join(accounts_file))
        .expect("Unable to read created file");

    insta::assert_snapshot!(contents);
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

    snapbox.assert().stderr_eq(formatdoc! {r"
        Command: account import
        Error: Incorrect class hash {} for account address {} was provided
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

    snapbox.assert().stderr_eq(indoc! {r"
        Command: account import
        Error: Class with hash 0x101 is not declared, try using --class-hash with a hash of the declared class
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

    snapbox.assert().stderr_eq(indoc! {r"
        Command: account import
        Error: Class hash for the account address 0x123 could not be found. Please provide the class hash
    "});
}

#[test_case("--url", URL, "with_url"; "with_url")]
#[test_case("--network", "devnet", "with_network"; "with_network")]
#[tokio::test]
pub async fn test_happy_case_add_profile(rpc_flag: &str, rpc_value: &str, case_name: &str) {
    let tempdir = tempdir().expect("Failed to create a temporary directory");
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "import",
        rpc_flag,
        rpc_value,
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

    let output = runner(&args).current_dir(tempdir.path()).assert();

    let config_path = Utf8PathBuf::from_path_buf(tempdir.path().join("snfoundry.toml"))
        .unwrap()
        .canonicalize_utf8()
        .unwrap();

    assert_stdout_contains(
        output,
        format!("Add Profile:  Profile my_account_import successfully added to {config_path}"),
    );

    let accounts_contents = fs::read_to_string(tempdir.path().join(accounts_file))
        .expect("Unable to read created file");
    let config_contents = fs::read_to_string(tempdir.path().join("snfoundry.toml"))
        .expect("Unable to read snfoundry.toml");

    set_snapshot_suffix!("{case_name}_accounts.json");
    insta::assert_snapshot!(accounts_contents);

    set_snapshot_suffix!("{case_name}_snfoundry.toml");
    insta::assert_snapshot!(config_contents);
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

    snapbox.assert().stdout_eq(indoc! {r"
        Success: Account imported successfully

        Account Name: my_account_import
    "});

    let contents = fs::read_to_string(tempdir.path().join(accounts_file))
        .expect("Unable to read created file");

    insta::assert_snapshot!(contents);
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

    snapbox.assert().stdout_eq(indoc! {r"
        Success: Account imported successfully

        Account Name: my_account_import
    "});

    let contents = fs::read_to_string(temp_dir.path().join(accounts_file))
        .expect("Unable to read created file");

    insta::assert_snapshot!(contents);
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
        "error: the argument '--private-key <PRIVATE_KEY>' cannot be used with '--private-key-file <PRIVATE_KEY_FILE_PATH>'",
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
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        formatdoc! {r"
        Command: account import
        Error: Failed to obtain private key from the file my_private_key

        Caused by:
            No such file or directory [..]
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
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: account import
        Error: Failed to obtain private key from the file my_private_key

        Caused by:
            failed to create Felt from string: invalid dec string
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

    insta::assert_snapshot!(contents);
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

    let output = runner(&args).current_dir(tempdir.path()).assert();

    let config_path = Utf8PathBuf::from_path_buf(tempdir.path().join("snfoundry.toml"))
        .unwrap()
        .canonicalize_utf8()
        .unwrap();

    assert_stdout_contains(
        output,
        format!("Add Profile:  Profile random successfully added to {config_path}"),
    );
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
        "0x3d8e70d1cbeca6eed8d4cf58fe812b24e741112730903dc91486afe9a5130cc",
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

    snapbox.assert().stdout_eq(indoc! {r"
        Success: Account imported successfully

        Account Name: my_account_import
    "});

    let contents = fs::read_to_string(tempdir.path().join(accounts_file))
        .expect("Unable to read created file");

    insta::assert_snapshot!(contents);
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
    let computed_address = "0x7298d9fc4bde13623bd53f4adb0110bd77ab5f3f3675402b5dfb418e149e56a";
    snapbox.assert().stderr_eq(formatdoc! {r"
        Command: account import
        Error: Computed address {computed_address} does not match the provided address 0x123. Please ensure that the provided salt, class hash, and account type are correct.
    "});
}

#[tokio::test]
pub async fn test_happy_case_default_name_generation() {
    let tempdir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts.json";

    let import_args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "import",
        "--url",
        URL,
        "--address",
        "0x123",
        "--private-key",
        "0x456",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH_CAIRO_0,
        "--type",
        "oz",
    ];

    let delete_args = vec![
        "--accounts-file",
        &accounts_file,
        "account",
        "delete",
        "--name",
        "account-2",
        "--network-name",
        "alpha-sepolia",
    ];

    for i in 0..3 {
        let snapbox = runner(&import_args).current_dir(tempdir.path());
        snapbox.assert().stdout_eq(formatdoc! {r"
        Success: Account imported successfully

        Account Name: account-{id}
    ", id = i + 1});
    }

    let contents = fs::read_to_string(tempdir.path().join(accounts_file))
        .expect("Unable to read created file");

    set_snapshot_suffix!("1_import");
    insta::assert_snapshot!(contents);

    let snapbox = runner(&delete_args).current_dir(tempdir.path()).stdin("Y");
    snapbox.assert().success().stdout_eq(indoc! {r"
        Success: Account deleted

        Account successfully removed
    "});

    let contents = fs::read_to_string(tempdir.path().join(accounts_file))
        .expect("Unable to read created file");

    set_snapshot_suffix!("2_delete");
    insta::assert_snapshot!(contents);

    let snapbox = runner(&import_args).current_dir(tempdir.path());
    snapbox.assert().stdout_eq(indoc! {r"
        Success: Account imported successfully

        Account Name: account-2
    "});

    let contents = fs::read_to_string(tempdir.path().join(accounts_file))
        .expect("Unable to read created file");

    set_snapshot_suffix!("3_import");
    insta::assert_snapshot!(contents);
}

#[tokio::test]
pub async fn test_import_with_address_alias() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_aliases.toml", None);
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "import",
        "--name",
        "my_account_import",
        "--address",
        "@map",
        "--private-key",
        "0x456",
        "--type",
        "oz",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().stdout_eq(indoc! {r"
        Success: Account imported successfully

        Account Name: my_account_import
    "});

    let contents = fs::read_to_string(tempdir.path().join(accounts_file))
        .expect("Unable to read created file");

    insta::assert_snapshot!(contents);
}

#[tokio::test]
pub async fn test_import_with_class_hash_alias() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_aliases.toml", None);
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "import",
        "--name",
        "my_account_import",
        "--address",
        "0x123",
        "--private-key",
        "0x456",
        "--class-hash",
        "@map-class",
        "--type",
        "oz",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());

    snapbox.assert().stdout_eq(indoc! {r"
        Success: Account imported successfully

        Account Name: my_account_import
    "});

    let contents = fs::read_to_string(tempdir.path().join(accounts_file))
        .expect("Unable to read created file");

    insta::assert_snapshot!(contents);
}

#[tokio::test]
pub async fn test_import_with_unknown_address_alias() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_aliases.toml", None);
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "import",
        "--name",
        "my_account_import",
        "--address",
        "@unknown",
        "--private-key",
        "0x456",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH_CAIRO_0,
        "--type",
        "oz",
    ];

    let output = runner(&args).current_dir(tempdir.path()).assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
            Command: account import
            Error: Invalid contract address

            Caused by:
                Alias `unknown` not found in config
        "},
    );
}

#[tokio::test]
pub async fn test_import_with_unknown_class_hash_alias() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_aliases.toml", None);
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "import",
        "--name",
        "my_account_import",
        "--address",
        "0x123",
        "--private-key",
        "0x456",
        "--class-hash",
        "@unknown",
        "--type",
        "oz",
    ];

    let output = runner(&args).current_dir(tempdir.path()).assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
            Command: account import
            Error: Invalid class hash

            Caused by:
                Alias `unknown` not found in config
        "},
    );
}

#[tokio::test]
pub async fn test_use_url_from_config() {
    let temp_dir = copy_config_to_tempdir("tests/data/files/snfoundry_correct.toml", None);
    let accounts_file = "accounts.json";
    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "import",
        "--address",
        DEVNET_PREDEPLOYED_ACCOUNT_ADDRESS,
        "--private-key",
        "0x456",
        "--type",
        "oz",
    ];

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(temp_dir.path());

    snapbox.assert().success();
}
