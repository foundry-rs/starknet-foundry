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

        // TODO(#3148): Once `scarb lint` can change warning to error, we should check status instead of checking if stdout is not empty
        // ATM `scarb lint` returns 0 even if there are warnings
        assert!(
            lint_output.stdout.is_empty(),
            "`scarb lint` output should be empty"
        );
    }
}

pub fn validate_cairo_lib(package_name: &str) {
    let package_path = project_root::get_project_root()
        .expect("Failed to get project root")
        .join(package_name);
    let package_path = package_path
        .canonicalize()
        .expect("Failed to canonicalize path");
    let package_path =
        Utf8PathBuf::from_path_buf(package_path).expect("Failed to convert to Utf8PathBuf");

    check_and_lint(&package_path);
}
