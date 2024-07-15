use indoc::formatdoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains, AsOutput};
use tempfile::tempdir;

use crate::{
    e2e::account::helpers::create_tempdir_with_accounts_file,
    helpers::{constants::URL, runner::runner},
};

#[tokio::test]
async fn test_happy_case() {
    let accounts_file_name = "temp_accounts.json";
    let temp_dir = create_tempdir_with_accounts_file(accounts_file_name, true).await;

    let accounts_file_path = temp_dir
        .path()
        .canonicalize()
        .expect("Unable to resolve a temporary directory path")
        .join(accounts_file_name);

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        &accounts_file_name,
        "account",
        "list",
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert!(output.as_stderr().is_empty());

    let expected = formatdoc!(
        "
        Available accounts (at {}):

        Network \"alpha-sepolia\":
        - user0:
        Account data:
          private key: 0x1e9038bdc68ce1d27d54205256988e85
          public key: 0x2f91ed13f8f0f7d39b942c80bfcd3d0967809d99e0cc083606cbe59033d2b39
          address: 0x4f5f24ceaae64434fa2bc2befd08976b51cf8f6a5d8257f7ec3616f61de263a
          type: OpenZeppelin

        Network \"custom-network\":
        - user3:
        Account data:
          private key: 0xe3e70682c2094cac629f6fbed82c07cd
          public key: 0x7e52885445756b313ea16849145363ccb73fb4ab0440dbac333cf9d13de82b9
          address: 0x7e00d496e324876bbc8531f2d9a82bf154d1a04a50218ee74cdd372f75a551a

        - user4:
        Account data:
          private key: 0x73fbb3c1eff11167598455d0408f3932e42c678bd8f7fbc6028c716867cc01f
          public key: 0x43a74f86b7e204f1ba081636c9d4015e1f54f5bb03a4ae8741602a15ffbb182
          address: 0x7ccdf182d27c7aaa2e733b94db4a3f7b28ff56336b34abf43c15e3a9edfbe91
          salt: 0x54aa715a5cff30ccf7845ad4659eb1dac5b730c2541263c358c7e3a4c4a8064
          deployed: true
        ",
        accounts_file_path.to_str().unwrap()
    );

    assert_stdout_contains(output, expected);
}

#[test]
fn test_accounts_file_does_not_exist() {
    let accounts_file_name = "some_inexistent_file.json";
    let temp_dir = tempdir().expect("Unable to create a temporary directory");

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        &accounts_file_name,
        "account",
        "list",
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().failure();

    assert!(output.as_stdout().is_empty());

    let expected = "Error: Accounts file = some_inexistent_file.json does not exist! \
        If you do not have an account create one with `account create` command \
        or if you're using a custom accounts file, make sure \
        to supply correct path to it with `--accounts-file` argument.";

    assert_stderr_contains(output, expected);
}

#[tokio::test]
async fn test_no_accounts_available() {
    let accounts_file_name = "temp_accounts.json";
    let temp_dir = create_tempdir_with_accounts_file(accounts_file_name, false).await;

    let accounts_file_path = temp_dir
        .path()
        .canonicalize()
        .expect("Unable to resolve a temporary directory path")
        .join(accounts_file_name);

    let args = vec![
        "--url",
        URL,
        "--accounts-file",
        &accounts_file_name,
        "account",
        "list",
    ];

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert!(output.as_stderr().is_empty());
    assert_stdout_contains(
        output,
        format!(
            "No accounts available at {}",
            accounts_file_path.to_str().unwrap()
        ),
    );
}
