use snapbox::cmd::{Command, cargo_bin};

#[must_use]
pub fn runner(args: &[&str]) -> Command {
    let command = Command::new(cargo_bin!("sncast")).args(args);
    command
}
