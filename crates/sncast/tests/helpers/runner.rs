use snapbox::cargo_bin;
use snapbox::cmd::Command;

#[must_use]
pub fn runner(args: &[&str]) -> Command {
    Command::new(cargo_bin!("sncast")).args(args)
}
