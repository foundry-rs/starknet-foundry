use snapbox::cmd::{cargo_bin, Command as SnapboxCommand};

pub fn runner() -> SnapboxCommand {
    let snapbox = SnapboxCommand::new(cargo_bin!("rust_test_runner"));
    snapbox
}
