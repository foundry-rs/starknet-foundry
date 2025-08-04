use crate::helpers::constants::{ACCOUNT_FILE_PATH, DEVNET_OZ_CLASS_HASH_CAIRO_0, URL};
use crate::helpers::fixtures::copy_file;
use crate::helpers::runner::runner;
use configuration::copy_config_to_tempdir;
use indoc::{formatdoc, indoc};

use crate::helpers::env::set_create_keystore_password_env;
use camino::Utf8PathBuf;
use conversions::string::IntoHexStr;
use serde_json::{json, to_string_pretty};
use shared::test_utils::output_assert::{
    AsOutput, assert_stderr_contains, assert_stdout, assert_stdout_contains,
};
use snapbox::assert_matches;
use sncast::AccountType;
use sncast::helpers::constants::{
    BRAAVOS_BASE_ACCOUNT_CLASS_HASH, BRAAVOS_CLASS_HASH, OZ_CLASS_HASH, READY_CLASS_HASH,
};
use std::fs;
use tempfile::tempdir;
use test_case::test_case;

#[test_case("oz"; "oz_account_type")]
#[test_case("ready"; "ready_account_type")]
#[test_case("braavos"; "braavos_account_type")]
#[tokio::test]
pub async fn test_happy_case(account_type: &str) {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "create",
        "--url",
        URL,
        "--name",
        "my_account",
        "--salt",
        "0x1",
        "--type",
        account_type,
    ];

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        indoc! {r"
        Success: Account created

        Address: 0x0[..]
        
        Account successfully created but it needs to be deployed. The estimated deployment fee is [..] STRK. Prefund the account to cover deployment transaction fee

        After prefunding the account, run:
        sncast --accounts-file accounts.json account deploy --url http://127.0.0.1:5055/rpc --name my_account

        To see account creation details, visit:
        account: [..]
        "},
    );

    let contents = fs::read_to_string(temp_dir.path().join(accounts_file))
        .expect("Unable to read created file");

    let expected = json!(
        {
            "alpha-sepolia": {
                "my_account": {
                    "address": "0x[..]",
                    "class_hash": "0x[..]",
                    "deployed": false,
                    "legacy": false,
                    "private_key": "0x[..]",
                    "public_key": "0x[..]",
                    "salt": "0x1",
                    "type": get_formatted_account_type(account_type)
                }
            }
        }
    );

    assert_matches(to_string_pretty(&expected).unwrap(), contents);
}

// TODO(#3556): Remove this test once we drop Argent account type
#[tokio::test]
pub async fn test_happy_case_argent_with_deprecation_warning() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "create",
        "--url",
        URL,
        "--name",
        "my_account",
        "--salt",
        "0x1",
        "--type",
        "argent",
    ];

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert_stdout(
        output,
        indoc! {r"
        [WARNING] Argent has rebranded as Ready. The `argent` option for the `--type` flag in `account create` is deprecated, please use `ready` instead.
        
        Success: Account created

        Address: 0x0[..]
        
        Account successfully created but it needs to be deployed. The estimated deployment fee is [..] STRK. Prefund the account to cover deployment transaction fee

        After prefunding the account, run:
        sncast --accounts-file accounts.json account deploy --url http://127.0.0.1:5055/rpc --name my_account

        To see account creation details, visit:
        account: [..]
        "},
    );

    let contents = fs::read_to_string(temp_dir.path().join(accounts_file))
        .expect("Unable to read created file");

    let expected = json!(
        {
            "alpha-sepolia": {
                "my_account": {
                    "address": "0x[..]",
                    "class_hash": "0x[..]",
                    "deployed": false,
                    "legacy": false,
                    "private_key": "0x[..]",
                    "public_key": "0x[..]",
                    "salt": "0x1",
                    "type": "ready"
                }
            }
        }
    );

    assert_matches(to_string_pretty(&expected).unwrap(), contents);
}

#[tokio::test]
pub async fn test_invalid_class_hash() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "create",
        "--url",
        URL,
        "--type",
        "oz",
        "--class-hash",
        "0x10101",
        "--name",
        "my_account_create_happy",
        "--salt",
        "0x1",
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: account create
        Error: Class with hash 0x10101 is not declared, try using --class-hash with a hash of the declared class
        "},
    );
}

#[tokio::test]
pub async fn test_happy_case_generate_salt() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "create",
        "--url",
        URL,
        "--name",
        "my_account",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH_CAIRO_0,
        "--type",
        "oz",
    ];

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(temp_dir.path());

    snapbox.assert().success().stdout_matches(indoc! {r"
        Success: Account created

        Address: 0x0[..]
        
        Account successfully created but it needs to be deployed. The estimated deployment fee is [..] STRK. Prefund the account to cover deployment transaction fee

        After prefunding the account, run:
        sncast --accounts-file accounts.json account deploy --url http://127.0.0.1:5055/rpc --name my_account

        To see account creation details, visit:
        account: [..]
        "});

    let contents = fs::read_to_string(temp_dir.path().join(accounts_file))
        .expect("Unable to read created file");
    assert!(contents.contains("my_account"));
    assert!(contents.contains("alpha-sepolia"));
    assert!(contents.contains("private_key"));
    assert!(contents.contains("public_key"));
    assert!(contents.contains("address"));
    assert!(contents.contains("salt"));
    assert!(contents.contains("class_hash"));
    assert!(contents.contains("legacy"));
    assert!(contents.contains("type"));
}

#[tokio::test]
pub async fn test_happy_case_add_profile() {
    let tempdir = tempdir().expect("Failed to create a temporary directory");
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "create",
        "--url",
        URL,
        "--name",
        "my_account",
        "--add-profile",
        "my_account",
    ];

    let output = runner(&args).current_dir(tempdir.path()).assert();
    let config_path = Utf8PathBuf::from_path_buf(tempdir.path().join("snfoundry.toml"))
        .unwrap()
        .canonicalize_utf8()
        .unwrap();

    assert_stdout_contains(
        output,
        format!("Add Profile: Profile my_account successfully added to {config_path}"),
    );

    let contents = fs::read_to_string(tempdir.path().join("snfoundry.toml"))
        .expect("Unable to read snfoundry.toml");
    assert!(contents.contains("[sncast.my_account]"));
    assert!(contents.contains("account = \"my_account\""));
}

#[tokio::test]
pub async fn test_happy_case_accounts_file_already_exists() {
    let accounts_file = "accounts.json";
    let temp_dir = tempdir().expect("Unable to create a temporary directory");

    copy_file(
        "tests/data/accounts/accounts.json",
        temp_dir.path().join(accounts_file),
    );
    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "create",
        "--url",
        URL,
        "--name",
        "my_account",
        "--salt",
        "0x1",
    ];

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(temp_dir.path());

    snapbox.assert().success().stdout_matches(indoc! {r"
        Success: Account created

        Address: 0x0[..]
        
        Account successfully created but it needs to be deployed. The estimated deployment fee is [..] STRK. Prefund the account to cover deployment transaction fee

        After prefunding the account, run:
        sncast --accounts-file accounts.json account deploy --url http://127.0.0.1:5055/rpc --name my_account

        To see account creation details, visit:
        account: [..]
        "});

    let contents = fs::read_to_string(temp_dir.path().join(accounts_file))
        .expect("Unable to read created file");
    assert!(contents.contains("my_account"));
    assert!(contents.contains("deployed"));
    assert!(contents.contains("legacy"));
}

#[tokio::test]
pub async fn test_add_profile_with_network() {
    let tempdir = tempdir().expect("Failed to create a temporary directory");
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "create",
        "--network",
        "sepolia",
        "--name",
        "my_account",
        "--add-profile",
        "my_account",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert();

    assert_stderr_contains(
        output,
        indoc! {r"
        error: the argument '--network <NETWORK>' cannot be used with '--add-profile <ADD_PROFILE>'
    "},
    );
}

#[tokio::test]
pub async fn test_profile_already_exists() {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None).unwrap();
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "create",
        "--url",
        URL,
        "--name",
        "myprofile",
        "--add-profile",
        "default",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: account create
        Error: Failed to add profile = default to the snfoundry.toml. Profile already exists
        "},
    );
}

#[tokio::test]
pub async fn test_account_already_exists() {
    let args = vec![
        "--accounts-file",
        ACCOUNT_FILE_PATH,
        "account",
        "create",
        "--url",
        URL,
        "--name",
        "user1",
        "--salt",
        "0x1",
    ];

    let snapbox = runner(&args);
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: account create
        Error: Account with name = user1 already exists in network with chain_id = SN_SEPOLIA
        "},
    );
}

#[test_case("oz"; "oz_account_type")]
#[test_case("ready"; "ready_account_type")]
#[test_case("braavos"; "braavos_account_type")]
#[tokio::test]
pub async fn test_happy_case_keystore(account_type: &str) {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let keystore_file = "my_key.json";
    let account_file = "my_account.json";
    set_create_keystore_password_env();

    let args = vec![
        "--keystore",
        keystore_file,
        "--account",
        account_file,
        "account",
        "create",
        "--url",
        URL,
        "--type",
        account_type,
    ];

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(temp_dir.path());

    snapbox.assert().stdout_matches(formatdoc! {r"
        Success: Account created

        Address: 0x0[..]
        
        Account successfully created but it needs to be deployed. The estimated deployment fee is [..] STRK. Prefund the account to cover deployment transaction fee

        After prefunding the account, run:
        sncast --account {} --keystore {} account deploy --url {}

        To see account creation details, visit:
        account: [..]
    ", account_file, keystore_file, URL});

    assert!(temp_dir.path().join(keystore_file).exists());

    let contents = fs::read_to_string(temp_dir.path().join(account_file))
        .expect("Unable to read created file");

    assert_matches(
        get_keystore_account_pattern(account_type.parse().unwrap(), None),
        contents,
    );
}

// TODO(#3556): Remove this test once we drop Argent account type
#[tokio::test]
pub async fn test_happy_case_keystore_argent_with_deprecation_warning() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let keystore_file = "my_key.json";
    let account_file = "my_account.json";
    set_create_keystore_password_env();

    let args = vec![
        "--keystore",
        keystore_file,
        "--account",
        account_file,
        "account",
        "create",
        "--url",
        URL,
        "--type",
        "argent",
    ];

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(temp_dir.path());

    snapbox.assert().stdout_matches(formatdoc! {r"
        [WARNING] Argent has rebranded as Ready. The `argent` option for the `--type` flag in `account create` is deprecated, please use `ready` instead.
        
        Success: Account created

        Address: 0x0[..]
        
        Account successfully created but it needs to be deployed. The estimated deployment fee is [..] STRK. Prefund the account to cover deployment transaction fee

        After prefunding the account, run:
        sncast --account {} --keystore {} account deploy --url {}

        To see account creation details, visit:
        account: [..]
    ", account_file, keystore_file, URL});

    assert!(temp_dir.path().join(keystore_file).exists());

    let contents = fs::read_to_string(temp_dir.path().join(account_file))
        .expect("Unable to read created file");

    assert_matches(
        get_keystore_account_pattern("argent".parse().unwrap(), None),
        contents,
    );
}

#[tokio::test]
pub async fn test_happy_case_keystore_add_profile() {
    let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None).unwrap();
    let keystore_file = "my_key.json";
    let account_file = "my_account.json";
    let accounts_json_file = "accounts.json";
    set_create_keystore_password_env();

    let args = vec![
        "--accounts-file",
        accounts_json_file,
        "--keystore",
        keystore_file,
        "--account",
        account_file,
        "account",
        "create",
        "--url",
        URL,
        "--add-profile",
        "with_keystore",
    ];

    let output = runner(&args).current_dir(tempdir.path()).assert();

    let config_path = Utf8PathBuf::from_path_buf(tempdir.path().join("snfoundry.toml"))
        .unwrap()
        .canonicalize_utf8()
        .unwrap();

    assert_stdout_contains(
        output,
        format!("Add Profile: Profile with_keystore successfully added to {config_path}"),
    );

    let contents =
        fs::read_to_string(tempdir.path().join(account_file)).expect("Unable to read created file");
    assert!(contents.contains("\"deployment\": {"));
    assert!(contents.contains("\"variant\": {"));
    assert!(contents.contains("\"version\": 1"));
    assert!(contents.contains("\"legacy\": false"));

    let contents = fs::read_to_string(tempdir.path().join("snfoundry.toml"))
        .expect("Unable to read snfoundry.toml");
    assert!(contents.contains(r"[sncast.with_keystore]"));
    assert!(contents.contains(r#"account = "my_account.json""#));
    assert!(!contents.contains(r#"accounts-file = "accounts.json""#));
    assert!(contents.contains(r#"keystore = "my_key.json""#));
    assert!(contents.contains(r#"url = "http://127.0.0.1:5055/rpc""#));
}

#[tokio::test]
pub async fn test_keystore_without_account() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let keystore_file = "my_key.json";

    set_create_keystore_password_env();

    let args = vec![
        "--keystore",
        keystore_file,
        "account",
        "create",
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: account create
        Error: Argument `--account` must be passed and be a path when using `--keystore`
        "},
    );
}

#[tokio::test]
pub async fn test_keystore_file_already_exists() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");

    let keystore_file = "my_key.json";
    let account_file = "my_account_new.json";

    copy_file(
        "tests/data/keystore/my_key.json",
        temp_dir.path().join(keystore_file),
    );
    set_create_keystore_password_env();

    let args = vec![
        "--keystore",
        keystore_file,
        "--account",
        account_file,
        "account",
        "create",
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: account create
        Error: Keystore file my_key.json already exists
        "},
    );
}

#[tokio::test]
pub async fn test_keystore_account_file_already_exists() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");

    let keystore_file = "my_key_new.json";
    let account_file = "my_account.json";

    copy_file(
        "tests/data/keystore/my_account.json",
        temp_dir.path().join(account_file),
    );

    set_create_keystore_password_env();

    let args = vec![
        "--keystore",
        keystore_file,
        "--account",
        account_file,
        "account",
        "create",
        "--url",
        URL,
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: account create
        Error: Account file my_account.json already exists
        "},
    );
}

#[tokio::test]
pub async fn test_happy_case_keystore_int_format() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let keystore_file = "my_key_int.json";
    let account_file = "my_account_int.json";

    set_create_keystore_password_env();

    let args = vec![
        "--keystore",
        keystore_file,
        "--account",
        account_file,
        "account",
        "create",
        "--url",
        URL,
        "--class-hash",
        DEVNET_OZ_CLASS_HASH_CAIRO_0,
        "--type",
        "oz",
    ];

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(temp_dir.path());

    snapbox.assert().stdout_matches(formatdoc! {r"
        Success: Account created

        Address: [..]
        
        Account successfully created but it needs to be deployed. The estimated deployment fee is [..] STRK. Prefund the account to cover deployment transaction fee

        After prefunding the account, run:
        sncast --account {} --keystore {} account deploy --url {}

        To see account creation details, visit:
        account: [..]
    ", account_file, keystore_file, URL});

    let contents = fs::read_to_string(temp_dir.path().join(account_file))
        .expect("Unable to read created file");
    assert!(contents.contains("\"deployment\": {"));
    assert!(contents.contains("\"variant\": {"));
    assert!(contents.contains("\"version\": 1"));
    assert!(contents.contains("\"legacy\": true"));
}

#[tokio::test]
pub async fn test_happy_case_default_name_generation() {
    let tempdir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts.json";

    let create_args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "create",
        "--url",
        URL,
        "--salt",
        "0x1",
    ];

    let delete_args = vec![
        "--accounts-file",
        &accounts_file,
        "account",
        "delete",
        "--name",
        "account-2",
        "--network",
        "sepolia",
    ];

    let assert_account_created = |id: usize| {
        runner(&create_args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(tempdir.path())
        .assert()
        .stdout_matches(formatdoc! {r"
            Success: Account created

            Address: 0x0[..]
            
            Account successfully created but it needs to be deployed. The estimated deployment fee is [..] STRK. Prefund the account to cover deployment transaction fee

            After prefunding the account, run:
            sncast --accounts-file accounts.json account deploy --url http://127.0.0.1:5055/rpc --name account-{id}

            To see account creation details, visit:
            account: [..]
        ", id = id});
    };

    for i in 0..3 {
        assert_account_created(i + 1);
    }

    let contents = fs::read_to_string(tempdir.path().join(accounts_file))
        .expect("Unable to read created file");

    assert!(
        ["account-1", "account-2", "account-3"]
            .iter()
            .all(|a| contents.contains(a))
    );

    runner(&delete_args)
        .current_dir(tempdir.path())
        .stdin("Y")
        .assert()
        .success()
        .stdout_matches(indoc! {r"
        Success: Account deleted

        Account successfully removed
    "});

    let contents_after_delete = fs::read_to_string(tempdir.path().join(accounts_file))
        .expect("Unable to read created file");

    assert!(!contents_after_delete.contains("account-2"));

    assert_account_created(2);

    let contents = fs::read_to_string(tempdir.path().join(accounts_file))
        .expect("Unable to read created file");

    assert!(contents.contains("account-2"));

    let expected = json!(
        {
            "alpha-sepolia": {
                "account-1": {
                    "address": "0x[..]",
                    "class_hash": "0x[..]",
                    "deployed": false,
                    "legacy": false,
                    "private_key": "0x[..]",
                    "public_key": "0x[..]",
                    "salt": "0x1",
                    "type": "open_zeppelin"
                },
                "account-2": {
                    "address": "0x[..]",
                    "class_hash": "0x[..]",
                    "deployed": false,
                    "legacy": false,
                    "private_key": "0x[..]",
                    "public_key": "0x[..]",
                    "salt": "0x1",
                    "type": "open_zeppelin"
                },
                "account-3": {
                    "address": "0x[..]",
                    "class_hash": "0x[..]",
                    "deployed": false,
                    "legacy": false,
                    "private_key": "0x[..]",
                    "public_key": "0x[..]",
                    "salt": "0x1",
                    "type": "open_zeppelin"
                },
            }
        }
    );

    assert_matches(to_string_pretty(&expected).unwrap(), contents);
}

fn get_formatted_account_type(account_type: &str) -> &str {
    match account_type {
        "oz" => "open_zeppelin",
        _ => account_type,
    }
}

fn get_keystore_account_pattern(account_type: AccountType, class_hash: Option<&str>) -> String {
    let account_json = match account_type {
        AccountType::OpenZeppelin => {
            json!(
                {
                    "version": 1,
                    "variant": {
                        "type": "open_zeppelin",
                        "version": 1,
                        "public_key": "0x[..]",
                        "legacy": false,
                    },
                    "deployment": {
                        "status": "undeployed",
                        "class_hash": class_hash.unwrap_or(&OZ_CLASS_HASH.into_hex_string()),
                        "salt": "0x[..]",
                    }
                }
            )
        }
        AccountType::Argent | AccountType::Ready => {
            json!(
                {
                    "version": 1,
                    "variant": {
                        // TODO(#3556): Remove hardcoded "argent" and use format! with `AccountType::Ready`
                        "type": "argent",
                        "version": 1,
                        "owner": "0x[..]",
                        "guardian": "0x0"
                    },
                    "deployment": {
                        "status": "undeployed",
                        "class_hash": class_hash.unwrap_or(&READY_CLASS_HASH.into_hex_string()),
                        "salt": "0x[..]",
                    }
                }
            )
        }
        AccountType::Braavos => {
            json!(
                {
                  "version": 1,
                  "variant": {
                    "type": "braavos",
                    "version": 1,
                    "multisig": {
                      "status": "off"
                    },
                    "signers": [
                      {
                        "type": "stark",
                        "public_key": "0x[..]"
                      }
                    ]
                  },
                  "deployment": {
                    "status": "undeployed",
                    "class_hash": class_hash.unwrap_or(&BRAAVOS_CLASS_HASH.into_hex_string()),
                    "salt": "0x[..]",
                    "context": {
                      "variant": "braavos",
                      "base_account_class_hash": BRAAVOS_BASE_ACCOUNT_CLASS_HASH
                    }
                  }
                }
            )
        }
    };

    to_string_pretty(&account_json).unwrap()
}

#[tokio::test]
pub async fn test_happy_case_deployment_fee_message() {
    let tempdir = tempdir().expect("Failed to create a temporary directory");

    let args = vec!["account", "create", "--url", URL];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        "Account successfully created but it needs to be deployed. The estimated deployment fee is 0.000836288000000000 STRK. Prefund the account to cover deployment transaction fee",
    );
}

#[tokio::test]
pub async fn test_happy_case_default_name_generation_when_accounts_file_empty() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "test_accounts.json";
    let accounts_path = temp_dir.path().join(accounts_file);
    std::fs::File::create(&accounts_path).expect("Failed to create empty accounts file");

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "create",
        "--url",
        URL,
        "--class-hash",
        DEVNET_OZ_CLASS_HASH_CAIRO_0,
        "--type",
        "oz",
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());

    snapbox.assert().success();
}

#[tokio::test]
pub async fn test_happy_case_accounts_file_empty() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "test_accounts.json";
    let accounts_path = temp_dir.path().join(accounts_file);
    std::fs::File::create(&accounts_path).expect("Failed to create empty accounts file");

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "create",
        "--url",
        URL,
        "--name",
        "my_account",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH_CAIRO_0,
        "--type",
        "oz",
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());

    snapbox.assert().success();
}

#[tokio::test]
pub async fn test_json_output_format() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "--json",
        "account",
        "create",
        "--url",
        URL,
        "--name",
        "my_account",
        "--salt",
        "0x1",
        "--type",
        "oz",
        "--add-profile",
        "my_account",
    ];

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(temp_dir.path());
    snapbox.assert().stdout_matches(indoc! {r#"
        {"add_profile":"Profile my_account successfully added to [..]/snfoundry.toml","address":"0x[..]","estimated_fee":"[..]","message":"Account successfully created but it needs to be deployed. The estimated deployment fee is [..] STRK. Prefund the account to cover deployment transaction fee/n/nAfter prefunding the account, run:/nsncast --accounts-file accounts.json account deploy --url [..] --name my_account"}
        {"links":"account: https://sepolia.starkscan.co/contract/0x[..]","title":"account creation"}
    "#});
}

#[tokio::test]
pub async fn test_no_explorer_links_on_localhost() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "create",
        "--url",
        "http://127.0.0.1:5055/rpc",
        "--name",
        "my_account",
        "--salt",
        "0x1",
        "--type",
        "oz",
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert!(
        !output
            .as_stdout()
            .contains("To see account creation details, visit:")
    );
}
