use camino::Utf8PathBuf;
use scarb_api::ScarbCommand;
use std::process::Stdio;

pub fn check_and_lint(package_path: &Utf8PathBuf) {
    let check_output = scarb()
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

    let lint_output = scarb()
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

fn scarb() -> ScarbCommand {
    let mut cmd = ScarbCommand::new();
    cmd.env("SCARB_IGNORE_CAIRO_VERSION", "true");
    cmd
}
