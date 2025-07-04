use super::common::runner::{setup_package, snforge_test_bin_path};
use snapbox::cmd::{Command as SnapboxCommand, OutputAssert};

fn runner_color(value: &str) -> SnapboxCommand {
    SnapboxCommand::new(snforge_test_bin_path())
        .arg("test")
        .arg("--color")
        .arg(value)
}

fn is_colored(output: &OutputAssert) -> bool {
    String::from_utf8(output.get_output().stdout.clone())
        .unwrap()
        .contains("\x1b[")
}

#[test]
fn color_always() {
    let temp = setup_package("simple_package");
    let snapbox = runner_color("always");
    let output = snapbox.current_dir(&temp).assert().code(1);
    assert!(
        is_colored(&output),
        "output expected to be colored but it is not"
    );
}

#[test]
#[ignore = "TODO(2356): Fix this test, compiling of snforge_scarb_plugin is causing issues, but only in CI"]
fn color_never() {
    let temp = setup_package("simple_package");
    let snapbox = runner_color("never");
    let output = snapbox.current_dir(&temp).assert().code(1);
    assert!(
        !is_colored(&output),
        "output not expected to be colored but it is"
    );
}
