use crate::helpers::runner::runner;

#[test]
fn test_exit_code_success() {
    let args = vec!["show-config"];
    let snapbox = runner(&args);
    snapbox.assert().success();
}

#[test]
fn test_exit_code_invalid_command() {
    let args = vec!["invalid-command"];
    let snapbox = runner(&args);
    let output = snapbox.assert().code(2);
    assert!(String::from_utf8_lossy(&output.get_output().stderr)
        .contains("unrecognized subcommand"));
}

#[test]
fn test_exit_code_rpc_error() {
    let args = vec![
        "call",
        "--url",
        "http://invalid-url-that-does-not-exist",
        "--contract-address",
        "0x0",
        "--function",
        "test",
    ];
    let snapbox = runner(&args);
    snapbox.assert().code(1);
}

#[test]
fn test_exit_code_missing_required_arg() {
    let args = vec!["call"];
    let snapbox = runner(&args);
    let output = snapbox.assert().code(2);
    assert!(String::from_utf8_lossy(&output.get_output().stderr)
        .contains("required arguments were not provided"));
}
