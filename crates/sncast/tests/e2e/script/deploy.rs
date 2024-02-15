use crate::helpers::constants::{ACCOUNT_FILE_PATH, SCRIPTS_DIR, URL};
use crate::helpers::fixtures::{duplicate_script_directory, get_accounts_path};
use crate::helpers::runner::runner;
use indoc::indoc;

#[tokio::test]
async fn test_with_calldata() {
    let tempdir =
        duplicate_script_directory(SCRIPTS_DIR.to_owned() + "/deploy", Vec::<String>::new());
    let accounts_json_path = get_accounts_path(ACCOUNT_FILE_PATH);

    let script_name = "with_calldata";
    let args = vec![
        "--accounts-file",
        accounts_json_path.as_str(),
        "--account",
        "user4",
        "--url",
        URL,
        "script",
        &script_name,
    ];

    let snapbox = runner(&args).current_dir(tempdir.path());
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        command: script
        status: success
    "});
}
