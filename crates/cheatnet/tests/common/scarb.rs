use scarb_api::ScarbCommand;

#[must_use]
pub fn scarb() -> ScarbCommand {
    let mut cmd = ScarbCommand::new();
    cmd.env("SCARB_IGNORE_CAIRO_VERSION", "true");
    cmd
}
