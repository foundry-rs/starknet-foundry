use snapbox::cmd::{cargo_bin, Command};
use std::path::Path;

#[must_use]
pub fn runner(args: &[&str], current_dir: Option<&Path>) -> Command {
    let command = Command::new(cargo_bin!("sncast")).args(args);
    let command = match current_dir {
        Some(dir) => command.current_dir(dir),
        None => command,
    };
    command
}
