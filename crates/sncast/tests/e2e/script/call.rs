use crate::helpers::constants::{SCRIPTS_DIR, URL};
use indoc::indoc;
use snapbox::cmd::{cargo_bin, Command};

#[tokio::test]
async fn test_happy_case() {
    let script_name = "call_happy";
    let args = vec!["--url", URL, "script", &script_name];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/misc")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        command: script
        status: success
    "});
}

#[tokio::test]
async fn test_failing() {
    let script_name = "call_fail";
    let args = vec!["--url", URL, "script", &script_name];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/misc")
        .args(args);
    snapbox.assert().success().stderr_matches(indoc! {r"
        command: script
        error: Got an exception while executing a hint: Custom Hint Error: An error occurred in the called contract [..]
    "});
}
