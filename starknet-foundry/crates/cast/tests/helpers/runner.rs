use snapbox::cmd::{cargo_bin, Command};

#[must_use]
pub fn runner(args: &[&str]) -> Command {
    let command = Command::new(cargo_bin!("cast")).args(args);
    command
}
