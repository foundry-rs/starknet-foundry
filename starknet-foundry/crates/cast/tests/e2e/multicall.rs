use crate::helpers::fixtures::default_cli_args;
use crate::helpers::runner::runner;
use std::path::Path;

#[tokio::test]
async fn test_happy_case() {
    
    let mut args = default_cli_args();

    let path = project_root::get_project_root().expect("failed to get project root path");

    let path = Path::new(&path).join("tests/data/multicall_configs/deploy_invoke.toml");
    let path_str = path.to_str().expect("failed converting path to str");

    args.append(&mut vec![
        "multicall",
        "--path",
        path_str,
    ]);

    let snapbox = runner(&args);
    let bdg = snapbox.assert();
    let out = bdg.get_output();

    assert!(out.stderr.len() == 0);
    let stdout_str = std::str::from_utf8(&out.stdout).expect("failed to convert command output to string");
    assert!(stdout_str.contains("transaction_hash: 0x1c243a35a959ed8c3112f25990b0380ed12102c9cd549001d49ad38e9b8de24"));

}
