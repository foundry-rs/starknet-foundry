use crate::helpers::constants::{
    ACCOUNT_FILE_PATH, DEVNET_OZ_CLASS_HASH_CAIRO_0, DEVNET_OZ_CLASS_HASH_CAIRO_1, URL,
};
use crate::helpers::fixtures::copy_file;
use crate::helpers::insta::set_snapshot_suffix;
use crate::helpers::runner::runner;
use configuration::test_utils::copy_config_to_tempdir;
use indoc::{formatdoc, indoc};

use camino::Utf8PathBuf;
use conversions::string::IntoHexStr;
use shared::test_utils::output_assert::{AsOutput, assert_stderr_contains, assert_stdout_contains};
use starknet_curve::curve_params::EC_ORDER;
use std::fs;
use tempfile::tempdir;
use test_case::test_case;

#[test_case("oz"; "open_zeppelin")]
#[test_case("ready"; "ready")]
#[test_case("braavos"; "braavos")]
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

    output.stdout_eq(indoc!(r"
        Success: Account created

        Address: 0x0[..]

        Account successfully created but it needs to be deployed. The estimated deployment fee is [..] STRK. Prefund the account to cover deployment transaction fee

        After prefunding the account, run:
        sncast --accounts-file accounts.json account deploy --url http://127.0.0.1:5055/rpc --name my_account

        To see account creation details, visit:
        account: [..]
    "));

    let accounts_json_contents = fs::read_to_string(temp_dir.path().join("accounts.json"))
        .expect("Unable to read accounts.json");

    let accounts_json: serde_json::Value =
        serde_json::from_str(&accounts_json_contents).expect("Failed to parse accounts.json");

    set_snapshot_suffix!("{account_type}");
    insta::assert_json_snapshot!(accounts_json, {
        ".**.address" => "[value]",
        ".**.class_hash" => "[value]",
        ".**.public_key" => "[value]",
        ".**.private_key" => "[value]",
    });
}

#[tokio::test]
pub async fn test_create_with_private_key() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts.json";
    let class_hash = DEVNET_OZ_CLASS_HASH_CAIRO_1.into_hex_string();

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
        "oz",
        "--class-hash",
        &class_hash,
        "--private-key",
        "0x456",
    ];

    let output = runner(&args)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    output.stdout_eq(indoc!(r"
        Success: Account created

        Address: 0x0[..]

        Account successfully created but it needs to be deployed. The estimated deployment fee is [..] STRK. Prefund the account to cover deployment transaction fee

        After prefunding the account, run:
        sncast --accounts-file accounts.json account deploy --url http://127.0.0.1:5055/rpc --name my_account
    "));

    let accounts_json_contents = fs::read_to_string(temp_dir.path().join("accounts.json"))
        .expect("Unable to read accounts.json");

    let accounts_json: serde_json::Value =
        serde_json::from_str(&accounts_json_contents).expect("Failed to parse accounts.json");

    insta::assert_json_snapshot!(accounts_json, {
        ".**.address" => "[value]",
        ".**.class_hash" => "[value]",
        ".**.salt" => "[value]",
    });
}

#[tokio::test]
pub async fn test_create_with_private_key_from_file() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts.json";
    let private_key_file = "my_private_key";
    let class_hash = DEVNET_OZ_CLASS_HASH_CAIRO_1.into_hex_string();

    fs::write(temp_dir.path().join(private_key_file), "0x456").unwrap();

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
        "oz",
        "--class-hash",
        &class_hash,
        "--private-key-file",
        private_key_file,
    ];

    let output = runner(&args)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    output.stdout_eq(indoc!(r"
        Success: Account created

        Address: 0x0[..]

        Account successfully created but it needs to be deployed. The estimated deployment fee is [..] STRK. Prefund the account to cover deployment transaction fee

        After prefunding the account, run:
        sncast --accounts-file accounts.json account deploy --url http://127.0.0.1:5055/rpc --name my_account
    "));

    let accounts_json_contents = fs::read_to_string(temp_dir.path().join("accounts.json"))
        .expect("Unable to read accounts.json");

    let accounts_json: serde_json::Value =
        serde_json::from_str(&accounts_json_contents).expect("Failed to parse accounts.json");

    insta::assert_json_snapshot!(accounts_json, {
        ".**.address" => "[value]",
        ".**.class_hash" => "[value]",
        ".**.salt" => "[value]",
    });
}

#[tokio::test]
pub async fn test_create_accept_only_one_private_key() {
    let args = vec![
        "account",
        "create",
        "--private-key",
        "0x456",
        "--private-key-file",
        "my_private_key",
    ];

    let output = runner(&args).assert().failure();

    assert_stderr_contains(
        output,
        "error: the argument '--private-key <PRIVATE_KEY>' cannot be used with '--private-key-file <PRIVATE_KEY_FILE_PATH>'",
    );
}

#[tokio::test]
pub async fn test_create_invalid_private_key_file_path() {
    let args = vec![
        "account",
        "create",
        "--url",
        URL,
        "--private-key-file",
        "my_private_key",
    ];

    let output = runner(&args).assert().failure();

    assert_stderr_contains(
        output,
        formatdoc! {r"
        Command: account create
        Error: Failed to obtain private key from the file my_private_key

        Caused by:
            No such file or directory [..]
        "},
    );
}

#[tokio::test]
pub async fn test_create_with_zero_private_key() {
    let args = vec![
        "account",
        "create",
        "--url",
        URL,
        "--name",
        "my_account",
        "--private-key",
        "0x0",
    ];

    let output = runner(&args).assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: account create
        Error: Invalid private key: the private key cannot be 0
        "},
    );
}

#[tokio::test]
pub async fn test_create_with_private_key_exceeding_curve_order() {
    // Equal to the STARK curve order, which is the first invalid value.
    let curve_order = EC_ORDER.into_hex_string();

    let args = vec![
        "account",
        "create",
        "--url",
        URL,
        "--name",
        "my_account",
        "--private-key",
        curve_order.as_str(),
    ];

    let output = runner(&args).assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: account create
        Error: Invalid private key: the private key must be smaller than the STARK curve order [..]
        "},
    );
}

#[tokio::test]
pub async fn test_create_with_class_hash_alias() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_aliases.toml", None);
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "create",
        "--name",
        "my_account",
        "--class-hash",
        "@oz-devnet",
        "--type",
        "oz",
    ];

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(tempdir.path());

    snapbox.assert().success().stdout_eq(indoc! {r"
        Success: Account created

        Address: 0x0[..]

        Account successfully created but it needs to be deployed. The estimated deployment fee is [..] STRK. Prefund the account to cover deployment transaction fee

        After prefunding the account, run:
        sncast --accounts-file accounts.json account deploy  --name my_account

        To see account creation details, visit:
        account: [..]
        "});

    let accounts_json_contents = fs::read_to_string(tempdir.path().join("accounts.json"))
        .expect("Unable to read accounts.json");

    let accounts_json: serde_json::Value =
        serde_json::from_str(&accounts_json_contents).expect("Failed to parse accounts.json");

    insta::assert_json_snapshot!(accounts_json, {
        ".**.address" => "[value]",
        ".**.class_hash" => "[value]",
        ".**.public_key" => "[value]",
        ".**.private_key" => "[value]",
        ".**.salt" => "[value]",
    });
}

#[tokio::test]
pub async fn test_create_with_unknown_class_hash_alias() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_aliases.toml", None);
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "create",
        "--name",
        "my_account",
        "--class-hash",
        "@unknown",
        "--type",
        "oz",
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: account create
        Error: Invalid class hash

        Caused by:
            Alias `unknown` not found in config
        "},
    );
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
    let output = snapbox.assert().failure();

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

    snapbox.assert().success().stdout_eq(indoc! {r"
        Success: Account created

        Address: 0x0[..]

        Account successfully created but it needs to be deployed. The estimated deployment fee is [..] STRK. Prefund the account to cover deployment transaction fee

        After prefunding the account, run:
        sncast --accounts-file accounts.json account deploy --url http://127.0.0.1:5055/rpc --name my_account

        To see account creation details, visit:
        account: [..]
        "});

    let accounts_json_contents = fs::read_to_string(temp_dir.path().join("accounts.json"))
        .expect("Unable to read accounts.json");

    let accounts_json: serde_json::Value =
        serde_json::from_str(&accounts_json_contents).expect("Failed to parse accounts.json");

    insta::assert_json_snapshot!(accounts_json, {
        ".**.address" => "[value]",
        ".**.class_hash" => "[value]",
        ".**.public_key" => "[value]",
        ".**.private_key" => "[value]",
        ".**.salt" => "[value]",
    });
}

#[test_case("--url", URL, "with_url"; "with_url")]
#[test_case("--network", "devnet", "with_network"; "with_network")]
#[tokio::test]
pub async fn test_happy_case_add_profile(rpc_flag: &str, rpc_value: &str, case_name: &str) {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_correct.toml", None);
    let accounts_file = "accounts.json";

    let args = vec![
        "--accounts-file",
        accounts_file,
        "account",
        "create",
        rpc_flag,
        rpc_value,
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

    let accounts_json_contents = fs::read_to_string(tempdir.path().join("accounts.json"))
        .expect("Unable to read accounts.json");

    let accounts_json: serde_json::Value =
        serde_json::from_str(&accounts_json_contents).expect("Failed to parse accounts.json");

    set_snapshot_suffix!("{case_name}_accounts.json");
    insta::assert_json_snapshot!(accounts_json, {
        ".**.address" => "[value]",
        ".**.class_hash" => "[value]",
        ".**.public_key" => "[value]",
        ".**.private_key" => "[value]",
        ".**.salt" => "[value]",
    });

    let snfoundry_toml_contents = fs::read_to_string(tempdir.path().join("snfoundry.toml"))
        .expect("Unable to read snfoundry.toml");

    set_snapshot_suffix!("{case_name}_snfoundry.toml");
    insta::assert_snapshot!(snfoundry_toml_contents);
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

    snapbox.assert().success().stdout_eq(indoc! {r"
        [WARNING] Accounts file was migrated to schema version 2; the original was saved as a .v1.bak file
        Success: Account created

        Address: 0x0[..]

        Account successfully created but it needs to be deployed. The estimated deployment fee is [..] STRK. Prefund the account to cover deployment transaction fee

        After prefunding the account, run:
        sncast --accounts-file accounts.json account deploy --url http://127.0.0.1:5055/rpc --name my_account

        To see account creation details, visit:
        account: [..]
        "});

    let accounts_json_contents = fs::read_to_string(temp_dir.path().join("accounts.json"))
        .expect("Unable to read accounts.json");

    let accounts_json: serde_json::Value =
        serde_json::from_str(&accounts_json_contents).expect("Failed to parse accounts.json");

    insta::assert_json_snapshot!(accounts_json, {
        ".**.address" => "[value]",
        ".**.class_hash" => "[value]",
        ".**.public_key" => "[value]",
        ".**.private_key" => "[value]",
        ".**.salt" => "[value]",
    });
}

#[tokio::test]
pub async fn test_profile_already_exists() {
    let tempdir = copy_config_to_tempdir("tests/data/files/snfoundry_correct.toml", None);
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
    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: account create
        Error: account `user1` already exists on network `alpha-sepolia`
        "},
    );
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
        .stdout_eq(formatdoc! {r"
            Success: Account created

            Address: 0x0[..]

            Account successfully created but it needs to be deployed. The estimated deployment fee is [..] STRK. Prefund the account to cover deployment transaction fee

            After prefunding the account, run:
            sncast --accounts-file accounts.json account deploy --url http://127.0.0.1:5055/rpc --name account-{id}

            To see account creation details, visit:
            account: [..]
        "});
    };

    for i in 0..3 {
        assert_account_created(i + 1);
    }

    let accounts_json_contents = fs::read_to_string(tempdir.path().join("accounts.json"))
        .expect("Unable to read accounts.json");

    let accounts_json: serde_json::Value =
        serde_json::from_str(&accounts_json_contents).expect("Failed to parse accounts.json");

    set_snapshot_suffix!("1_create");
    insta::assert_json_snapshot!(accounts_json, {
        ".**.address" => "[value]",
        ".**.class_hash" => "[value]",
        ".**.public_key" => "[value]",
        ".**.private_key" => "[value]",
        ".**.salt" => "[value]",
    });

    let output = runner(&delete_args)
        .current_dir(tempdir.path())
        .stdin("Y")
        .assert()
        .success();

    output.stdout_eq(indoc! {r"
        Success: Account deleted

        Account successfully removed
    "});

    let accounts_json_contents = fs::read_to_string(tempdir.path().join("accounts.json"))
        .expect("Unable to read accounts.json");

    let accounts_json: serde_json::Value =
        serde_json::from_str(&accounts_json_contents).expect("Failed to parse accounts.json");

    set_snapshot_suffix!("2_delete");
    insta::assert_json_snapshot!(accounts_json, {
        ".**.address" => "[value]",
        ".**.class_hash" => "[value]",
        ".**.public_key" => "[value]",
        ".**.private_key" => "[value]",
        ".**.salt" => "[value]",
    });

    assert_account_created(2);

    let accounts_json_contents = fs::read_to_string(tempdir.path().join("accounts.json"))
        .expect("Unable to read accounts.json");

    let accounts_json: serde_json::Value =
        serde_json::from_str(&accounts_json_contents).expect("Failed to parse accounts.json");

    set_snapshot_suffix!("3_create");
    insta::assert_json_snapshot!(accounts_json, {
        ".**.address" => "[value]",
        ".**.class_hash" => "[value]",
        ".**.public_key" => "[value]",
        ".**.private_key" => "[value]",
        ".**.salt" => "[value]",
    });
}

#[tokio::test]
pub async fn test_happy_case_default_name_generation_when_accounts_file_empty() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts.json";
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

    let output = runner(&args)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    output.stdout_eq(indoc!(r"
        Success: Account created

        Address: 0x0[..]

        Account successfully created but it needs to be deployed. The estimated deployment fee is [..] STRK. Prefund the account to cover deployment transaction fee

        After prefunding the account, run:
        sncast --accounts-file accounts.json account deploy --url http://127.0.0.1:5055/rpc --name account-1
    "));

    let accounts_json_contents =
        fs::read_to_string(accounts_path).expect("Unable to read accounts.json");

    let accounts_json: serde_json::Value =
        serde_json::from_str(&accounts_json_contents).expect("Failed to parse accounts.json");

    insta::assert_json_snapshot!(accounts_json, {
        ".**.address" => "[value]",
        ".**.class_hash" => "[value]",
        ".**.public_key" => "[value]",
        ".**.private_key" => "[value]",
        ".**.salt" => "[value]",
    });
}

#[tokio::test]
pub async fn test_happy_case_deployment_fee_message() {
    let tempdir = tempdir().expect("Failed to create a temporary directory");

    let args = vec!["account", "create", "--url", URL];

    let snapbox = runner(&args).current_dir(tempdir.path());
    let output = snapbox.assert().success();

    output.stdout_eq(indoc!("
        Success: Account created

        Address: 0x03cf60d8427f4e36b52dc18d5eefab9781d17887f9a18df49915896b95870922

        Account successfully created but it needs to be deployed. The estimated deployment fee is 0.002242288000000000 STRK. Prefund the account to cover deployment transaction fee

        After prefunding the account, run:
        sncast account deploy --url http://127.0.0.1:5055/rpc --name account-9
    "));

    let accounts_json_contents = fs::read_to_string(tempdir.path().join("accounts.json"))
        .expect("Unable to read accounts.json");

    let accounts_json: serde_json::Value =
        serde_json::from_str(&accounts_json_contents).expect("Failed to parse accounts.json");

    insta::assert_json_snapshot!(accounts_json, {
        ".**.address" => "[value]",
        ".**.class_hash" => "[value]",
        ".**.public_key" => "[value]",
        ".**.private_key" => "[value]",
        ".**.salt" => "[value]",
    });
}

#[tokio::test]
pub async fn test_happy_case_accounts_file_empty() {
    let temp_dir = tempdir().expect("Unable to create a temporary directory");
    let accounts_file = "accounts.json";
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

    let output = runner(&args)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    output.stdout_eq(indoc!("
        Success: Account created

        Address: 0x0[..]

        Account successfully created but it needs to be deployed. The estimated deployment fee is [..] STRK. Prefund the account to cover deployment transaction fee

        After prefunding the account, run:
        sncast --accounts-file accounts.json account deploy --url http://127.0.0.1:5055/rpc --name my_account
    "));

    let accounts_json_contents =
        fs::read_to_string(accounts_path).expect("Unable to read accounts.json");

    let accounts_json: serde_json::Value =
        serde_json::from_str(&accounts_json_contents).expect("Failed to parse accounts.json");

    insta::assert_json_snapshot!(accounts_json, {
        ".**.address" => "[value]",
        ".**.public_key" => "[value]",
        ".**.private_key" => "[value]",
        ".**.salt" => "[value]",
    });
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
    snapbox.assert().stdout_eq(indoc! {r#"
        {"add_profile":"Profile my_account successfully added to [..]/snfoundry.toml","address":"0x[..]","command":"account create","estimated_fee":"[..]","message":"Account successfully created but it needs to be deployed. The estimated deployment fee is [..] STRK. Prefund the account to cover deployment transaction fee/n/nAfter prefunding the account, run:/nsncast --accounts-file accounts.json account deploy --url [..] --name my_account","type":"response"}
        {"links":"account: https://sepolia.voyager.online/contract/0x[..]","title":"account creation","type":"notification"}
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

// TODO(#4027): Fix either test or underlying bug
#[tokio::test]
pub async fn test_use_url_from_config() {
    let accounts_file = "accounts.json";
    let temp_dir = copy_config_to_tempdir("tests/data/files/snfoundry_correct.toml", None);
    copy_file(
        "tests/data/accounts/accounts.json",
        temp_dir.path().join(accounts_file),
    );

    let args = vec!["--accounts-file", accounts_file, "account", "create"];

    let snapbox = runner(&args)
        .env("SNCAST_FORCE_SHOW_EXPLORER_LINKS", "1")
        .current_dir(temp_dir.path());

    snapbox.assert().success();
}
