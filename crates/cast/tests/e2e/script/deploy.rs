use crate::helpers::constants::{SCRIPTS_DIR, URL};
use crate::helpers::runner::runner;
use indoc::indoc;
use std::path::Path;

#[tokio::test]
async fn test_with_calldata() {
    let script_name = "with_calldata";
    let args = vec![
        "--accounts-file",
        "../../accounts/accounts.json",
        "--account",
        "user4",
        "--url",
        URL,
        "script",
        &script_name,
    ];

    let current_dir = Path::new(&SCRIPTS_DIR).join("deploy");
    let snapbox = runner(&args, Some(current_dir.as_path()));

    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        command: script
        status: success
    "});
}
