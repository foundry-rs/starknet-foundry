use crate::helpers::constants::{SCRIPTS_DIR, URL};
use crate::helpers::runner::runner;
use indoc::indoc;

use std::path::Path;

#[tokio::test]
async fn test_missing_field() {
    let script_name = "missing_field";
    let args = vec![
        "--accounts-file",
        "../../../accounts/accounts.json",
        "--account",
        "user4",
        "--url",
        URL,
        "script",
        &script_name,
    ];

    let current_dir = Path::new(&SCRIPTS_DIR).join("declare/missing_field");
    let snapbox = runner(&args, Some(current_dir.as_path()));

    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        error: Wrong number of arguments. Expected 2, found: 1
        ...
    "});
}

#[tokio::test]
async fn test_wrong_contract_name() {
    let script_name = "no_contract";
    let args = vec![
        "--accounts-file",
        "../../../accounts/accounts.json",
        "--account",
        "user4",
        "--url",
        URL,
        "script",
        &script_name,
    ];

    let current_dir = Path::new(&SCRIPTS_DIR).join("declare/no_contract");
    let snapbox = runner(&args, Some(current_dir.as_path()));

    snapbox.assert().success().stderr_matches(indoc! {r"
        command: script
        error: Got an exception while executing a hint: [..]
    "});
}
