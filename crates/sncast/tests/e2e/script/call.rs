use crate::helpers::constants::{SCRIPTS_DIR, URL};
use crate::helpers::fixtures::duplicate_script_directory;
use crate::helpers::runner::runner;
use indoc::indoc;

#[tokio::test]
async fn test_happy_case() {
    let tempdir =
        duplicate_script_directory(SCRIPTS_DIR.to_owned() + "/misc", Vec::<String>::new());

    let script_name = "call_happy";
    let args = vec!["--url", URL, "script", &script_name];

    let snapbox = runner(&args).current_dir(tempdir.path());
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        command: script
        status: success
    "});
}

#[tokio::test]
async fn test_failing() {
    let tempdir =
        duplicate_script_directory(SCRIPTS_DIR.to_owned() + "/misc", Vec::<String>::new());

    let script_name = "call_fail";
    let args = vec!["--url", URL, "script", &script_name];

    let snapbox = runner(&args).current_dir(tempdir.path());
    snapbox.assert().success().stderr_matches(indoc! {r"
        command: script
        error: Got an exception while executing a hint: Hint Error: An error [..]Entry point EntryPointSelector(StarkFelt[..]not found in contract[..]
    "});
}
