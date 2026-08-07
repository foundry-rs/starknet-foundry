use scarb_api::ScarbCommand;

#[must_use]
pub fn scarb() -> ScarbCommand {
    let mut cmd = ScarbCommand::new();
    cmd.env("SCARB_IGNORE_CAIRO_VERSION", "true");
    cmd
}

#[must_use]
pub fn scarb_with_stdio() -> ScarbCommand {
    let mut cmd = scarb();
    cmd.inherit_stderr();
    cmd.inherit_stdout();
    cmd
}
