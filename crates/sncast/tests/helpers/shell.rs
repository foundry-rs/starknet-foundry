use camino::Utf8PathBuf;
use snapbox::cmd::Command;

#[must_use]
pub fn os_specific_shell(script_path: &Utf8PathBuf) -> Command {
    let test_path = script_path.with_extension("sh");
    let absolute_test_path = test_path.canonicalize_utf8().unwrap();

    Command::new(absolute_test_path)
}
