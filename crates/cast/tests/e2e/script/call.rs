use crate::helpers::constants::{SCRIPTS_DIR, URL};
use crate::helpers::runner::runner;
use indoc::indoc;
use std::path::Path;

#[tokio::test]
async fn test_happy_case() {
    let script_name = "call_happy";
    let args = vec![
        "--accounts-file",
        "../../accounts/accounts.json",
        "--account",
        "user1",
        "--url",
        URL,
        "script",
        &script_name,
    ];

    let current_dir = Path::new(&SCRIPTS_DIR).join("misc");
    let snapbox = runner(&args, Some(current_dir.as_path()));

    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        command: script
        status: success
    "});
}

#[tokio::test]
async fn test_failing() {
    let script_name = "call_fail";
    let args = vec![
        "--accounts-file",
        "../../accounts/accounts.json",
        "--account",
        "user1",
        "--url",
        URL,
        "script",
        &script_name,
    ];

    let current_dir = Path::new(&SCRIPTS_DIR).join("misc");
    let snapbox = runner(&args, Some(current_dir.as_path()));

    snapbox.assert().success().stderr_matches(indoc! {r"
        command: script
        error: Got an exception while executing a hint: Hint Error: Entry point [..] not found in contract.
    "});
}
