use super::common::runner::setup_package;
use snapbox::cmd::{cargo_bin, Command as SnapboxCommand, OutputAssert};

fn runner_color(value: &str) -> SnapboxCommand {
    SnapboxCommand::new(cargo_bin!("snforge"))
        .arg("test")
        .arg("--color")
        .arg(value)
}

fn is_colored(output: &OutputAssert) -> bool {
    String::from_utf8(output.get_output().stdout.clone())
        .unwrap()
        .contains("\x1b[")
}

// Join this two tests together so that they won't overwrite each other's ENV
#[test]
fn color_always_and_never() {
    let temp = setup_package("simple_package");
    let snapbox = runner_color("always");
    let output = snapbox.current_dir(&temp).assert().code(1);
    assert!(
        is_colored(&output),
        "output expected to be colored but it is not"
    );

    let temp = setup_package("simple_package");
    let snapbox = runner_color("never");
    let output = snapbox.current_dir(&temp).assert().code(1);
    assert!(
        !is_colored(&output),
        "output not expected to be colored but it is"
    );
}
