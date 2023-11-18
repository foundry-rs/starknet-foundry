use crate::helpers::constants::{SCRIPTS_DIR, URL};
use indoc::indoc;
use snapbox::cmd::{cargo_bin, Command};

#[tokio::test]
async fn test_happy_case() {
    let script_path = "src/deploy_happy.cairo";
    let args = vec![
        "--accounts-file",
        "../../../accounts/accounts.json",
        "--account",
        "user4",
        "--url",
        URL,
        "script",
        &script_path,
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/deploy/script") // todo: change after #271
        .args(args);

    snapbox.assert().success().stdout_matches(indoc! {r"
        [..]
        [..]
        command: script
        status: success
    "});
}
