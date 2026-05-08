use crate::helpers::runner::{Cast, runner};
use clap::ValueEnum;
use clap_complete::Shell;
use configuration::test_utils::copy_config_to_tempdir;
use indoc::formatdoc;
use shared::test_utils::output_assert::{AsOutput, assert_stderr_contains};
use tempfile::tempdir;

#[test]
fn test_happy_case() {
    for variant in Shell::value_variants() {
        let shell = variant.to_string();
        let args = vec!["completions", shell.as_str()];

        let snapbox = runner(&args);

        snapbox.assert().success();
    }
}

#[test]
fn test_generate_completions_unsupported_shell() {
    // SAFETY: Tests run in parallel and share the same environment variables.
    // However, this modification is applies only to this one test.
    unsafe {
        std::env::set_var("SHELL", "/bin/unsupported");
    }
    let args = vec!["completions"];

    let snapbox = runner(&args);

    let output = snapbox.assert().failure();

    assert_stderr_contains(
        output,
        formatdoc!(
            r"
            Error: Unsupported shell
            "
        ),
    );
}

#[test]
fn test_completions_invalid_local_config() {
    let t = copy_config_to_tempdir("tests/data/files/snfoundry_malformed.toml", None);
    let args = vec!["completions", "bash"];

    let snapbox = runner(&args).current_dir(t.path());

    let output = snapbox.assert().success();
    assert!(output.as_stdout().starts_with("_sncast() {"));
}

#[test]
fn test_completions_invalid_global_config() {
    let global_dir = copy_config_to_tempdir("tests/data/files/snfoundry_malformed.toml", None);
    let local_dir = tempdir().unwrap();
    let args = vec!["completions", "bash"];

    let snapbox = Cast::new()
        .config_dir(global_dir.path())
        .command()
        .args(&args)
        .current_dir(local_dir.path());

    let output = snapbox.assert().success();
    assert!(output.as_stdout().starts_with("_sncast() {"));
}
