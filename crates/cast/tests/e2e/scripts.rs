use crate::helpers::constants::{SCRIPTS_DIR, URL};
use indoc::indoc;
use snapbox::cmd::{cargo_bin, Command};

#[tokio::test]
async fn test_happy_case() {
    let script_name = "hello_world";
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

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/hello_world")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        command: script
        status: success
    "});
}

#[tokio::test]
async fn test_call_failing() {
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

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/hello_world")
        .args(args);
    snapbox.assert().success().stderr_matches(indoc! {r"
        command: script
        error: Got an exception while executing a hint: Hint Error: Entry point [..] not found in contract.
    "});
}

#[tokio::test]
async fn test_run_script_from_different_directory() {
    let script_name = "hello_world";
    let path_to_scarb_toml = "hello_world/Scarb.toml";
    let args = vec![
        "--accounts-file",
        "../accounts/accounts.json",
        "--account",
        "user1",
        "--url",
        URL,
        "--path-to-scarb-toml",
        path_to_scarb_toml,
        "script",
        &script_name,
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR)
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        command: script
        status: success
    "});
}

#[tokio::test]
async fn test_run_script_from_different_directory_no_path_to_scarb_toml() {
    let script_name = "hello_world";
    let args = vec![
        "--accounts-file",
        "../accounts/accounts.json",
        "--account",
        "user1",
        "--url",
        URL,
        "script",
        &script_name,
    ];

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR)
        .args(args);
    snapbox.assert().success().stderr_matches(indoc! {r"
        ...
        command: script
        error: Path [..]Scarb.toml does not exist
    "});
}

#[tokio::test]
async fn test_verify_imports_within_same_package() {
    let script_name = "verify_import";
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

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/hello_world")
        .args(args);
    snapbox.assert().success().stdout_matches(indoc! {r"
        ...
        command: script
        status: success
    "});
}

#[tokio::test]
async fn test_fail_when_using_starknet_syscall() {
    let script_name = "using_starknet_syscall";
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

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(SCRIPTS_DIR.to_owned() + "/hello_world")
        .args(args);
    snapbox.assert().success().stderr_matches(indoc! {r"
        ...
        command: script
        error: Got an exception while executing a hint: Hint Error: Starknet syscalls are not supported
    "});
}
