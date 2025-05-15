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

    // TODO(#3149)
    if cfg!(feature = "scarb_since_2_10") {
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

        // TODO(#3212): Once out minimal supported scarb version is 2.12.0, we should
        // check status instead of checking if stdout is not empty
        assert!(
            lint_output.stdout.is_empty(),
            "`scarb lint` output should be empty"
        );
    }
}
