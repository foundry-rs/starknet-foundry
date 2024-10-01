use clap::{Arg, Command as ClapCommand};
use snapbox::cmd::{cargo_bin, Command};

#[must_use]
pub fn runner(args: &[&str]) -> Command {
    let clap_command = ClapCommand::new(cargo_bin!("sncast").to_str().unwrap())
        .args(args.iter().map(|&s| Arg::new(s.to_string())));
    clap_command.debug_assert();
    let command = Command::new(cargo_bin!("sncast")).args(args);
    command
}
