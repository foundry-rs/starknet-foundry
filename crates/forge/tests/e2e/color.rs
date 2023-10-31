use snapbox::cmd::{cargo_bin, Command as SnapboxCommand, OutputAssert};

use crate::e2e::common::runner::setup_package;

fn runner_color(value: &str) -> SnapboxCommand {
    SnapboxCommand::new(cargo_bin!("snforge"))
        .arg("test")
        .arg("--color")
        .arg(value)
}

fn has_escape(output: &OutputAssert) -> bool {
    String::from_utf8(output.get_output().stdout.clone())
        .unwrap()
        .contains("\x1b[")
}

#[test]
fn color_always() {
    let temp = setup_package("simple_package");
    let snapbox = runner_color("always");
    let output = snapbox.current_dir(&temp).assert().code(1);
    assert!(has_escape(&output), "Stdout doesn't contains escape code");
}

#[test]
fn color_never() {
    let temp = setup_package("simple_package");
    let snapbox = runner_color("never");
    let output = snapbox.current_dir(&temp).assert().code(1);
    assert!(!has_escape(&output), "Stdout contains escape code");
}
