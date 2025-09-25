use camino::Utf8PathBuf;
use scarb_api::ScarbCommand;
use std::process::Stdio;

pub fn check_and_lint(package_path: &Utf8PathBuf) {
    let check_output = ScarbCommand::new()
        .current_dir(package_path)
        .arg("check")
        .command()
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("Failed to run `scarb check`");
    assert!(
        check_output.status.success(),
        "`scarb check` failed in {package_path}",
    );

    let lint_output = ScarbCommand::new()
        .current_dir(package_path)
        .arg("lint")
        .command()
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("Failed to run `scarb lint`");
    assert!(
        lint_output.status.success(),
        "`scarb lint` failed in {package_path}"
    );
}
