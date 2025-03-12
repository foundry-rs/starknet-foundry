use crate::e2e::common::runner::snforge_test_bin_path;
use clap::ValueEnum;
use clap_complete::Shell;
use indoc::formatdoc;
use shared::test_utils::output_assert::assert_stdout_contains;
use snapbox::cmd::Command;

#[test]
fn test_happy_case() {
    for variant in Shell::value_variants() {
        let shell = variant.to_string();
        let snapbox = Command::new(snforge_test_bin_path())
            .arg("completion")
            .arg(shell.as_str());

        snapbox.assert().success();
    }
}

#[test]
fn test_generate_completions_unsupported_shell() {
    // SAFETY: Tests run in parallel and share the same environment variables.
    // However, this modification applies only to this one test.
    unsafe {
        std::env::set_var("SHELL", "/bin/unsupported");
    }

    let snapbox = Command::new(snforge_test_bin_path()).arg("completion");

    let output = snapbox.assert().failure();

    assert_stdout_contains(
        output,
        formatdoc!(
            r"
            [ERROR] Unsupported shell
            "
        ),
    );
}
