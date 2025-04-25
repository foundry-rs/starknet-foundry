use camino::Utf8PathBuf;
use snapbox::cmd::Command;

#[must_use]
pub fn os_specific_shell(script_path: &Utf8PathBuf) -> Command {
    let script_extension = if cfg!(windows) { "ps1" } else { "sh" };
    let test_path = script_path.with_extension(script_extension);

    if cfg!(windows) {
        Command::new("powershell")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-File")
            .arg(test_path)
    } else {
        Command::new(test_path)
    }
}
