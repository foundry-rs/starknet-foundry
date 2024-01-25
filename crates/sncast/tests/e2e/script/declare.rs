use crate::helpers::constants::{SCRIPTS_DIR, URL};
use indoc::indoc;
use snapbox::cmd::{cargo_bin, Command};

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

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/declare/missing_field")
        .args(args);
    snapbox.assert().failure().stdout_matches(indoc! {r"
        ...
        error: Wrong number of arguments. Expected 3, found: 2
        ...
    "});
}
